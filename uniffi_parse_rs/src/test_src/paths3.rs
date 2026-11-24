use url::Url;

#[derive(uniffi::Record)]
struct RemoteRecord { }

uniffi::custom_type!(Url, String, {
    remote,
    lower: |url| url.to_string(),
    try_lift: |s| Url::parse(s),
});
