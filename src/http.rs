#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum StatusCode {
    Continue,           //100
    SwitchingProtocols, //101
    Processing,         //102
    EarlyHints,         //103

    OK,                          //200
    Created,                     //201
    Accepted,                    //202
    NonAuthoritativeInformation, //203
    NoContent,                   //204
    ResetContent,                //205
    PartialContent,              //206

    MovedPermanently,  //301
    Found,             //302
    SeeOther,          //303
    NotModified,       //304
    TemporaryRedirect, //307
    PermanentRedirect, //308

    BadRequest,                  //400
    Unauthorized,                //401
    PaymentRequired,             //402
    Forbidden,                   //403
    NotFound,                    //404
    MethodNotAllowed,            //405
    NotAcceptable,               //406
    ProxyAuthenticationRequired, //407
    RequestTimeout,              //408
    Conflict,                    //409
    Gone,                        //410
    LengthRequired,              //411
    PreconditionFailed,          //412
    ContentTooLarge,             //413
    URITooLong,                  //414
    UnsupportedMediaType,        //415
    TooManyRequests,             //429

    InternalServerError,     //500
    NotImplemented,          //501
    BadGateway,              //502
    ServiceUnavailable,      //503
    GatewayTimeout,          //504
    HTTPVersionNotSupported, //505
}
