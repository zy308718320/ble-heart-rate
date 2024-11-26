import * as bleHeartRate from './index.js';

// const bleHeartRate = new BleHeartRate();

bleHeartRate.start((device_id, heart_rate) => {
  console.log('result', device_id, heart_rate);
});