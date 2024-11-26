import * as bleHeartRate from './index.js';

bleHeartRate.start((device_id, heart_rate) => {
  console.log('result', device_id, heart_rate);
});