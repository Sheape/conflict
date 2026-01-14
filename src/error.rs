use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    SSLConflict(#[from] SslProviderConflict),
}

#[derive(Debug, Error, Diagnostic)]
#[error("SSL provider conflict")]
#[diagnostic(
    code(conflict::ssl_provider),
    severity(Error),
    help("Remove either OpenSSL or BoringSSL dependency.")
)]
pub struct SslProviderConflict {
    #[source_code]
    pub manifest: NamedSource<String>,

    #[label("OpenSSL is pulled in here")]
    pub openssl_span: SourceSpan,

    #[label("BoringSSL is pulled in here")]
    pub boringssl_span: SourceSpan,
}
