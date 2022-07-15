const socket = new WebSocket("ws:localhost:3000/api/ws");
const sse = new EventSource("/api/sse");
sse.onmessage = function (event) {
    console.log('SSE ', JSON.parse(event.data))
}
socket.addEventListener('open', function (event) {
    socket.send(JSON.stringify({ k: 'View', t: 'All' }))
});
socket.addEventListener('message', function (event) {
    console.log('WS ', JSON.parse(event.data));
});

let program = null;
const spawn = document.getElementById('spawn').addEventListener('click', () => {
    fetch('/api/spawn', { method: 'POST', body: JSON.stringify({ program }), headers: {'Content-Type': 'application/json'} })
})
document.getElementById('compile').addEventListener('click', () => {
    const code = document.getElementById('code').files[0];
    fetch('/api/compile', { method: 'POST', body: code }).then(res => res.json()).then(p => {
        program = p
        document.getElementById('spawn').disabled = false
        document.getElementById('spawn').textContent = 'Spawn ' + program
    })
})
