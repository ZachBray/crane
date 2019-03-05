use std::str::FromStr;
use failure::Error;
use rusoto_s3::{S3, S3Client, PutObjectRequest};
use rusoto_core::Region;
use rusoto_core::ByteStream;

pub struct Bucket {
    region: String,
    bucket: String,
    key_prefix: String,
}

impl Bucket {
    pub fn new(region: String, bucket: String, key_prefix: String) -> Self {
        Bucket {
            region,
            bucket,
            key_prefix,
        }
    }

    pub fn put(&self, key: &str, content: Vec<u8>) -> Result<String, Error> {
        let region = Region::from_str(&self.region)?;
        let client = S3Client::new(region);
        let body: ByteStream = ByteStream::from(content);
        client.put_object(PutObjectRequest {
            acl: None,
            body: Some(body),
            bucket: self.bucket.to_string(),
            cache_control: None,
            content_disposition: None,
            content_encoding: None,
            content_language: None,
            content_length: None,
            content_md5: None,
            content_type: None,
            expires: None,
            grant_full_control: None,
            grant_read: None,
            grant_read_acp: None,
            grant_write_acp: None,
            key: format!("{}/{}", &self.key_prefix, &key),
            metadata: None,
            request_payer: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
            sse_customer_key_md5: None,
            ssekms_key_id: None,
            server_side_encryption: None,
            storage_class: None,
            tagging: None,
            website_redirect_location: None,
        }).sync()?;
        let url = format!("https://s3-{}.amazonaws.com/{}/{}/{}",
                          &self.region, &self.bucket, &self.key_prefix, &key);
        Ok(url)
    }
}
