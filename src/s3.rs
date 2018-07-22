use futures::Future;
use rusoto_core::{ProvideAwsCredentials, Region};
use rusoto_core::credential::{AwsCredentials, EnvironmentProvider};
use rusoto_s3::PutObjectRequest;
use rusoto_s3::util::PreSignedRequest;

pub struct S3Access {
    bucket: String,
    region: Region,
    creds: AwsCredentials,
    cdn_url: String,
}

impl S3Access {
    pub fn new(bucket: String, cdn_url: String) -> S3Access {
        let region = Region::default(); // reads from environment var
        let creds = EnvironmentProvider.credentials().wait()
            .expect("couldn't build AWS credentials");
        S3Access { bucket, region, creds, cdn_url }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    pub filename: String,
    file_type: String
}

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    url: String,
    directory: String,
    filename: String,
    get_url: String,
}

pub fn sign_upload(s3: &S3Access, directory: &str, req: UploadRequest) -> UploadResponse {
    let put_req = PutObjectRequest {
        bucket: s3.bucket.clone(),
        key: format!("{}/{}", directory, req.filename.clone()),
        content_type: Some(req.file_type.clone()),
        ..Default::default()
    };
    let url = put_req.get_presigned_url(&s3.region, &s3.creds);

    let get_url = format!("https://{}/{}/{}", &s3.cdn_url, directory, req.filename);
    UploadResponse {
        url,
        get_url,
        directory: directory.to_owned(),
        filename: req.filename.clone()
    }
}

#[cfg(test)]
impl UploadRequest {
    pub fn fake() -> UploadRequest {
        UploadRequest {
            filename: String::from("f"),
            file_type: String::from("t")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dotenv;
    use rand;
    use rusoto_core::credential::StaticProvider;
    use reqwest::{StatusCode, Client};
    use std::env;

    #[test]
    fn upload_unauthorized() {
        let creds = StaticProvider::new_minimal(String::from("foo"), String::from("baz"))
            .credentials().wait().expect("couldn't make static creds");
        let bucket = String::from("photothing-heroku-dev");
        let cdn_url = "foo.com".to_string();
        let access = S3Access { bucket, creds, region: Region::UsEast1, cdn_url };
        let req = UploadRequest {
            filename: String::from(""), file_type: String::from("")
        };

        let url = sign_upload(&access, "automation", req).url;
        assert!(url.starts_with("https://"));

        let client = Client::new();
        let res = client.put(&url)
            .body("some content")
            .send()
            .expect("request failed");
        assert_eq!(res.status(), StatusCode::Forbidden, "unathorized request didn't fail");
    }

    #[test]
    fn upload_integration_test() {
        dotenv::dotenv().ok();
        let creds = EnvironmentProvider.credentials().wait()
            .expect("couldn't build AWS credentials");
        let bucket = String::from("photothing-heroku-dev");
        let cdn_url = env::var("ROCKET_CDN_URL").expect("missing cdn url");
        let access = S3Access { bucket, creds, region: Region::UsEast1, cdn_url };
        let suffix: u8 = rand::random();
        let filename = format!("upload-{}.txt", suffix);
        let req = UploadRequest {
            filename: filename.clone(), file_type: String::from("text/plain")
        };

        let response = sign_upload(&access, "automation", req);
        let url = response.url;
        assert!(url.starts_with("https://"));
        assert!(response.get_url.find(&access.cdn_url).is_some());
        assert_eq!(response.filename, filename);
        assert_eq!(response.directory, "automation");

        // make sure we can upload content to the returned presigned url
        let content: i64 = rand::random();
        let body = format!("foobizbaz={}", content);
        let client = Client::new();
        let res = client.put(&url)
            .body(body.clone())
            .send()
            .expect("request failed");
        assert_eq!(res.status(), StatusCode::Ok, "upload request got 200 status");

        // and that we can fetch it back using the get_url
        let mut get = client.get(&response.get_url).send().expect("request failed");
        assert_eq!(get.status(), StatusCode::Ok, "got uploaded content");
        assert_eq!(get.text().expect("no text"), body, "got correct content back");
    }
}
