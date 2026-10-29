const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:7331');

ws.on('open', function open() {
  console.log('Connected');
  ws.send(JSON.stringify({
    type: 'hello',
    client: 'android',
    version: '1.0.0',
    device_name: 'Test Device'
  }));
  
  setTimeout(() => {
    console.log('Sending CallStarted event...');
    ws.send(JSON.stringify({
      type: 'call_started',
      number: '9380070210',
      call_id: 'test-call-123',
      client: 'android'
    }));
  }, 2000);
});

ws.on('message', function incoming(data) {
  console.log('Message from server: ', data.toString());
});

