use core::slice;
use dialoguer::Select;
use hex_literal::hex;
use pcsc::{
    Context, Error, Protocols, ReaderNames, ReaderState, Scope, ShareMode, State, MAX_ATR_SIZE,
    MAX_BUFFER_SIZE, PNP_NOTIFICATION,
};
use pcsc_sys::SCardControl;
use std::ffi::CStr;
use std::thread::current;
use std::time::Duration;
use std::{ffi::CString, fmt::format, result};
use tokio::io;

fn main() {
    //  Read Without Encriptの領域を全て吸い出してダンプ

    // データの吸い出し
    let ctx: Context = match Context::establish(Scope::User) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprint!("コンテキストの作成エラー: {}", err);
            std::process::exit(1);
        }
    };

    let mut buf: [u8; 2048] = [0; 2048];
    let mut readers = match ctx.list_readers(&mut buf) {
        Ok(readers) => readers,
        Err(err) => {
            eprintln!("リーダの検索エラー: {}", err);
            std::process::exit(1);
        }
    };

    // カードリーダの状態を監視するループ処理
    // TODO: 非同期でできそう？
    let mut rd_status = vec![ReaderState::new(PNP_NOTIFICATION(), State::CHANGED)];
    let rd: &CStr;
    'outer: loop {
        // 死んでるリーダを候補から外すメソッドと処理
        fn is_dead(rs: &ReaderState) -> bool {
            rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
        }
        for rs in &rd_status {
            if is_dead(rs) {
                println!("Removing {:?}", rs.name());
            }
        }
        rd_status.retain(|rs| !is_dead(rs));

        // 新規リーダーを登録
        for name in &mut readers {
            if !rd_status.iter().any(|rs| rs.name() == name) {
                rd_status.push(ReaderState::new(name, State::UNAWARE));
            }
        }

        //リーダの待機状態を更新
        for rs in &mut rd_status {
            rs.sync_current_state();
        }

        // 状態が変化するまで待機
        // 待機時間はDuration型で指定
        match ctx.get_status_change(Duration::from_secs(30), &mut rd_status) {
            Ok(()) => {}
            Err(Error::Timeout) => {
                eprintln!("タイムアウト");
                std::process::exit(1);
            }
            Err(err) => {
                panic!("不明なエラー: {:?}", err);
            }
        }

        // 現在の状態を表示
        for rs in &rd_status {
            if rs.name() != PNP_NOTIFICATION() {
                if rs.event_state().contains(State::PRESENT) {
                    rd = rs.name();
                    break 'outer;
                }
            }
        }
    }

    // カードリーダーとNFCカードを接続
    let card = match ctx.connect(rd, ShareMode::Shared, Protocols::ANY) {
        Ok(card) => card,
        Err(Error::NoSmartcard) => {
            println!("対応カードではありません");
            return;
        }
        Err(err) => {
            eprintln!("カードの読み取りに失敗しました: {}", err);
            std::process::exit(1);
        }
    };

    // カードからIDmを吸い出す
    let idm_cmd = hex!("FF CA 00 00");
    let mut buf = [0; MAX_BUFFER_SIZE];
    let res_apdu = match card.transmit(&idm_cmd, &mut buf) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("APDUコマンドの送信（IDm読み取り）に失敗: {}", err);
            std::process::exit(1);
        }
    };

    // IDm読み出し時、成功と失敗とで処理を分ける
    // レスポンスデータの末尾が16進数で '90 00' の場合が成功
    let res_len = res_apdu.len();
    let result_code = &res_apdu[res_len - 2..res_len];
    if !(*result_code.get(0).unwrap() == 0x90 && *result_code.get(1).unwrap() == 0x00) {
        println!("> IDmの読み出しに失敗");
    } else {
        println!("> IDm成功");
        println!("生データ\n{:?}", res_apdu);
        println!("16進数データ\n{:02X?}", res_apdu);
    }

    // TODO: ダンプさせる

    // TODO: 表示処理
}
