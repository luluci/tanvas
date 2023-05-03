// https://microsoft.github.io/windows-docs-rs/doc/windows/index.html
use windows::{
	core::*,
	Win32::{UI::WindowsAndMessaging::*, System::SystemServices::EVENTLOG_BACKWARDS_READ},
	Win32::System::{
		WindowsProgramming::{
			GetComputerNameW,
		}, 
	},
	Win32::System::EventLog::{
		EVENTLOGRECORD,
		EVENTLOG_SEQUENTIAL_READ,
		READ_EVENT_LOG_READ_FLAGS,
		OpenEventLogW,
		ReadEventLogW,
		CloseEventLog,
	},
};
use serde::{ Serialize, Deserialize };
use chrono::{self, TimeZone, Datelike, Timelike};

pub fn message_box(msg: &str) {
	unsafe {
		MessageBoxW(None, &HSTRING::from(msg), w!("MsgBoxCaption"), MB_OK);
	}
}

pub fn get_computer_name() -> String {
	match get_computer_name_raw() {
		Ok(ret) => String::from_utf16_lossy(&ret.0[..ret.1 as usize]),
		Err(_) => String::default(),
	}
}

fn get_computer_name_pcwstr() -> std::result::Result<PCWSTR, &'static str> {
	match get_computer_name_raw() {
		Ok(mut ret) => Ok(PCWSTR(ret.0.as_mut_ptr())),
		Err(err) => Err(err),
	}
}

fn get_computer_name_raw() -> std::result::Result<([u16; 256], u32), &'static str> {
	unsafe {
		// コンピュータ名取得
		const COMPUTER_NAME_BUFF_SIZE: u32 = 256;
		let mut computer_name_buff_size_raw: u32 = COMPUTER_NAME_BUFF_SIZE;
		let mut computer_name_buff: [u16; COMPUTER_NAME_BUFF_SIZE as usize] = [0; COMPUTER_NAME_BUFF_SIZE as usize];
		let result = GetComputerNameW(PWSTR(computer_name_buff.as_mut_ptr()), &mut computer_name_buff_size_raw);
		if result.as_bool() {
			Ok((computer_name_buff, computer_name_buff_size_raw))
		} else {
			println!("Failed to GetComputerNameW (get_computer_name_raw)");
			Err("")
		}
		// UTF-16 -> UTF-8
		//let computer_name = String::from_utf16_lossy(&computer_name_buff[..computer_name_buff_size_raw as usize]);
		// コンピュータ名作成
		//let computer_name = PCWSTR(computer_name_buff.as_mut_ptr());
	}
}

fn get_log_string(node: &EVENTLOGRECORD) -> String {
	unsafe {
		let mut ptr = node as *const EVENTLOGRECORD as *const char;
		ptr = ptr.offset(node.StringOffset as isize);
		println!("addr:{:?}", ptr);
		let mut str_size: isize = 0;
		while *ptr.offset(str_size) != '\0' {
			println!("  +1: addr:{:?} = {}", ptr.offset(str_size), *ptr.offset(str_size) as u8);
			str_size += 1;
		}
		println!("  +1: addr:{:?} = {}", ptr.offset(str_size), *ptr.offset(str_size) as u8);
		let slice = std::slice::from_raw_parts(ptr as *const u16, str_size as usize / 2);

		String::from_utf16_lossy(slice)
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Date {
	year: i32,
	month: u32,
	day: u32,
	hour: u32,
	minute: u32,
	second: u32,
	millisecond: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventLog {
    event_id: u32,
	content: String,
	time_generated: Date,
}

pub fn get_logon_logoff_log(size: i32) -> Vec<EventLog> {
	// コンピュータ名取得
	let computer_name = match get_computer_name_pcwstr() {
		Ok(str) => str,
		Err(msg) => {
			println!("Failed to GetComputerNameW(get_logon_logoff_log): {}", msg);
			return vec![];
		}
	};
	// EventLogソース指定
	let source = w!("System");

	let mut capacity: usize = 100;
	if size > 0 {
		capacity = size as usize;
	}

	let mut counter = 0;
	let func = |node: EVENTLOGRECORD| -> EventLogFuncResult<EventLog> {
		if counter > capacity {
			return EventLogFuncResult::Finish;
		}

		//
		let event_id = node.EventID & 0xFFFF;
		// DateTime作成
		let dt = match chrono::Local.timestamp_opt(node.TimeGenerated as i64, 0) {
			chrono::LocalResult::Single(t) => t,
			chrono::LocalResult::Ambiguous(_min, max) => max,
			chrono::LocalResult::None => {
				return EventLogFuncResult::Finish;
			},
		};
		//Ok(format!("ID:{:04X}, Code:{:02X} .", node.EventID, id))
		//Ok(format!("ID:{}. ", event_id))
		// EventLog作成
		let mut log = true;
		let content: String;
		match event_id {
			6005 => content = "ログオン:起動".into(),
			6006 => content = "ログオフ:正常シャットダウン".into(),
			6008 => content = "ログオフ:正常ではないシャットダウン".into(),
			6009 => content = "ログオン:起動時にブート情報を記録".into(),
			1 => content = "ログオフ:スリープ".into(),
			42 => content = "ログオン:スリープから復帰".into(),
			12 => content = "ログオン:OS起動".into(),
			13 => content = "ログオン:起動".into(),
			7001 => content = "ログオン:起動".into(),
			7002 => content = "ログオフ:シャットダウン".into(),
			_ => {
				content = String::default();
				log = false
			},
		}
		if log {
			counter += 1;
			EventLogFuncResult::Log(
				EventLog {
					event_id,
					content,
					time_generated: Date {
						year: dt.year(),
						month: dt.month0(),
						day: dt.day(),
						hour: dt.hour(),
						minute: dt.minute(),
						second: dt.second(),
						millisecond: 0,
					},
				}
			)
		} else {
			EventLogFuncResult::Continue
		}
	};

	get_event_log(computer_name, source, capacity, func)
}


pub enum EventLogFuncResult<T> {
    Finish,
    Continue,
    Log(T),
}

pub fn get_event_log<T, F>(computer_name: PCWSTR, source: PCWSTR, capacity: usize, mut f: F) -> Vec<T>
	where F: FnMut(EVENTLOGRECORD) -> EventLogFuncResult<T>
{
	let mut log = Vec::<T>::with_capacity(capacity);

	unsafe {
		// http://nienie.com/~masapico/api_sample_eventlog02_c.html
		// EventLog構造体サイズの取得からやるべき？

		/*
		// WindowsEventハンドルを取得
		let handle = match RegisterEventSourceW(None, w!("System")) {
			Ok(handle) => handle,
			Err(e) => {
				println!("{}", e);
				return;
			},
		};
		*/

		// EventLogハンドラ取得
		let handle = match OpenEventLogW(computer_name, source) {
			Ok(handle) => handle,
			Err(e) => {
				println!("{}", e);
				return log;
			},
		};
		// EventLog読み出しデータ作成
		let mut dummy_buff: [u8; 1] = [0; 1];
		let mut eventlog_buff = Vec::<u8>::new();
		let mut eventlog_buff_size: u32 = 256;
		eventlog_buff.resize(eventlog_buff_size as usize, 0);

		let mut read_log_size: u32 = 0;
		let mut needed_byte_size: u32 = 0;
		let read_flag = READ_EVENT_LOG_READ_FLAGS(EVENTLOG_BACKWARDS_READ + EVENTLOG_SEQUENTIAL_READ.0);

		let dur = std::time::Duration::from_micros(10000);

		// 繰り返し読みだす
		loop {
			std::thread::sleep(dur);
			//print!("");
			// EventLogサイズ判定
			// 1バイトバッファを指定する。
			// EventLogがバッファに収まらない場合はRESULTがfalseになるとともに、
			// needed_byte_sizeにEventLogのサイズが設定される。
			// EventLogごとにサイズが違うようなので1個ずつ判定して取り出す。
			let _ = ReadEventLogW(
				handle,
				read_flag,
				0,
				dummy_buff.as_mut_ptr() as *mut std::ffi::c_void,
				1,
				&mut read_log_size,
				&mut needed_byte_size);
			// バッファサイズが足りなければ拡張する
			if eventlog_buff_size < needed_byte_size {
				eventlog_buff_size = needed_byte_size;
				eventlog_buff.resize(eventlog_buff_size as usize, 0);
			}
			// EventLog読み出し
			let result = ReadEventLogW(
				handle,
				read_flag,
				0,
				eventlog_buff.as_mut_ptr() as *mut std::ffi::c_void,
				eventlog_buff_size as u32,
				&mut read_log_size,
				&mut needed_byte_size);
			// 読み出し失敗で終了
			if !result.as_bool() {
				break;
			}
			// 読み出したEventLogをクロージャがチェックする
			// チェック結果がfalseで終了
			// Okの場合はログとして格納しておくデータが入っている
			let node: *const EVENTLOGRECORD = eventlog_buff.as_ptr() as *const EVENTLOGRECORD;
			let check_result = match f(*node) {
				EventLogFuncResult::Log(node) => {
					log.push(node);
					true
				},
				EventLogFuncResult::Continue => true,
				EventLogFuncResult::Finish => {
					//println!("{}", e);
					false
				},
			};
			if !check_result {
				break;
			}
		}
		// EventLogを閉じる
		CloseEventLog(handle);
	}

	log
}



pub fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
