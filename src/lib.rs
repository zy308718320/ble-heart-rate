use napi::{
  tokio,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use futures::prelude::*;
use btleplug::api::{Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};

#[macro_use]
extern crate napi_derive;

async fn get_central(manager: &Manager) -> Adapter {
  let adapters = manager.adapters().await.unwrap();
  adapters.into_iter().nth(0).unwrap()
}

fn process_manufacturer_data(buff: &[u8], start_code: u8) -> Option<(u32, u32)> {
  if buff.is_empty() {
    return None;
  }

  let sliced_buff = if let Some(pos) = buff.iter().position(|&x| x == start_code) {
    &buff[pos..]
  } else {
    buff
  };

  if sliced_buff.get(0) == Some(&start_code) && sliced_buff.len() >= 6 {
    let device_bytes: [u8; 4] = sliced_buff[1..5]
      .try_into()
      .expect("slice with incorrect length");
    let device_id: u32 = u32::from_be_bytes(device_bytes);
    let heart_rate: u32 = sliced_buff[5].into();

    Some((device_id, heart_rate))
  } else {
    None
  }
}

#[napi(ts_args_type = "callback: (device_id: number, heart_rate: number) => void")]
pub async fn start(callback: ThreadsafeFunction<(u32, u32), (), (u32, u32), false>) {
  let manager = match Manager::new().await {
    Ok(m) => m,
    Err(_) => {
      return;
    }
  };

  let central = get_central(&manager).await;

  let central_state = match central.adapter_state().await {
    Ok(cs) => cs,
    Err(_) => {
      return;
    }
  };
  println!("CentralState: {:?}", central_state);

  let events = match central.events().await {
    Ok(e) => e,
    Err(_) => {
      return;
    }
  };

  match central.start_scan(ScanFilter::default()).await {
    Ok(_) => {
      println!("StartScan");
    }
    Err(_) => {
      return;
    }
  };

  let key: u16 = 0xFF04;
  let start_code: u8 = 0xA1;
  let empty_vec: Vec<u8> = Vec::new();

  let mut events = events;
  
  tokio::spawn(async move {
    while let Some(event) = events.next().await {
      match event {
        CentralEvent::ManufacturerDataAdvertisement {
          manufacturer_data,
          id: _,
        } => {
          let buff = manufacturer_data.get(&key).unwrap_or(&empty_vec);
          if let Some((device_id, heart_rate)) = process_manufacturer_data(buff, start_code) {
            callback.call((device_id, heart_rate), ThreadsafeFunctionCallMode::Blocking);
          }
        }
        _ => {}
      }
    }
  });
}
