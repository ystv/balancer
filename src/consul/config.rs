pub struct Config {
    /// The address of the consul server. This must include the protocol to connect over eg. http or https.
    pub address: String,
    // / The consul secret token to make authenticated requests to the consul server.
    // pub token: Option<String>
}
