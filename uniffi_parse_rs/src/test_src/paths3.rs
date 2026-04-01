use url::Url;

#[derive(uniffi::Record)]
struct RemoteRecord { }

uniffi::custom_type!(Url, String, {
    into: |url| url.to_string(),
    try_from: |s| Url::parse(s),
});
