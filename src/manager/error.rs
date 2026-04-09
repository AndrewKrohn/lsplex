#[derive(Debug)]
pub enum MiddlewareError {
    SendToSvrFail,
    SendToClientFail,
    LspCommFail,
    ClientFail,
}

#[derive(Debug)]
pub enum ClientError {
    FailedRecvMsg,
    FailedReadMsg,
    CommFail,
}

#[derive(Debug)]
pub enum LspError {
    FwdMsgError,
    StdioError,
    ReadMsgError,
    RecvMsgError,
}
