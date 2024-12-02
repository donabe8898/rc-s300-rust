use core::slice;
use dialoguer::Select;
use hex_literal::hex;
use std::vec;

use pcsc::{
    Context, Error, Protocols, ReaderNames, ReaderState, Scope, ShareMode, State, MAX_ATR_SIZE,
    MAX_BUFFER_SIZE, MAX_BUFFER_SIZE_EXTENDED, PNP_NOTIFICATION,
};
use pcsc_sys::SCardControl;
use std::ffi::CStr;
use std::thread::current;
use std::time::Duration;
use std::{error, ffi::CString, fmt::format, result};
use tokio::io;

// IDm関連の実装
pub struct IDm {
    idm: Result<Vec<u8>, pcsc::Error>,
}

impl IDm {
    // コンストラクタ
    pub fn new() -> Self {
        IDm {
            idm: Ok(Vec::new()),
        }
    }

    // カードからIDmを取得してstructへ格納
    pub fn get_idm(&mut self, card: &pcsc::Card) {
        let idm_cmd = hex!("FF CA 00 00 00"); // どういったカードなのかを知るコマンド
        let mut buf = [0; MAX_BUFFER_SIZE];

        match card.transmit(&idm_cmd, &mut buf) {
            Ok(res_apdu) => {
                let res_len = res_apdu.len();
                let result_code = &res_apdu[res_len - 2..res_len];

                if !(*result_code.get(0).unwrap() == 0x90 && *result_code.get(1).unwrap() == 0x00) {
                    self.idm = Err(pcsc::Error::InvalidAtr); // 適当にエラーを返す(無効な値)
                    println!("> IDmの読み出しに失敗");
                    return;
                } else {
                    self.idm = Ok(Vec::from(&res_apdu[0..8]))
                }
            }

            Err(err) => {
                eprintln!("APDUコマンドの送信（IDm読み取り）に失敗: {}", err);
                self.idm = Err(pcsc::Error::CommError); // 適当にエラーを返す2(通信エラー)
                return;
            }
        }
    }

    // 試しにicocaの残高を読み取ってみる
    pub fn print_bal(&mut self, card: &pcsc::Card) {
        // ICOCA属性情報のサービスコードは0x008B
        // http://jennychan.web.fc2.com/format/suica.html
        let mut buf = [0; MAX_BUFFER_SIZE];
        let select_file_cmd = hex!("FF A4 00 01 02 8B 00"); // サービスコードはリトルエディアンで指定
        let read_binary_cmd = hex!("FF B0 00 00 00");

        let r = match card.transmit(&select_file_cmd, &mut buf) {
            Ok(responce) => responce,

            Err(err) => {
                eprintln!("APDUコマンドの送信に失敗: {}", err);
                std::process::exit(1);
            }
        };
        println!("select file cmd: {:02X?}", r);

        let r = match card.transmit(&read_binary_cmd, &mut buf) {
            Ok(responce) => responce,

            Err(err) => {
                eprintln!("APDUコマンドの送信に失敗: {}", err);
                std::process::exit(1);
            }
        };
        println!("read binary: {:02X?}", r);

        // 残高表示
        // 10進数へ変換
        let comb = ((r[12] as u16) << 8) | (r[11] as u16);
        println!("💰残高: {} 円", comb);
    }

    // IDmを個別で返す
    pub fn resp_idm(&self) -> Result<Vec<u8>, pcsc::Error> {
        self.idm.clone()
    }
}
