use core::slice;
use dialoguer::Select;
use hex_literal::hex;
use pcsc::{
    Context, Error, Protocols, ReaderNames, ReaderState, Scope, ShareMode, State, MAX_ATR_SIZE,
    MAX_BUFFER_SIZE, PNP_NOTIFICATION,
};
use pcsc_sys::SCardControl;
use std::ffi::CStr;
use std::time::Duration;
use std::{ffi::CString, fmt::format, result};
use tokio::io;

fn main() {
    //  Read Without Encriptの領域を全て吸い出してダンプ
}
