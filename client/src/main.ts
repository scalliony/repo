import { Application, Container, Graphics, Texture, TilingSprite } from "pixi.js";

document.addEventListener('DOMContentLoaded', () => {
    const pixi = new Application({ resizeTo: document.body })
    document.body.appendChild(pixi.view)

    const tile = new TilingSprite(Texture.from(new URL(
        'data/tile.png',
        import.meta.url
    ).toString()), pixi.view.width, pixi.view.height)

    pixi.renderer.on('resize', () => {
        tile.width = pixi.view.width
        tile.height = pixi.view.height
    })
    pixi.ticker.add((dt) => {
        // tile.tilePosition.x -= 2*dt
        // tile.tilePosition.y -= dt
    })

    tile.interactive = true
    tile
        .on('mousedown', function (ev) {
            const startPosition = ev.data.getLocalPosition(this.parent);
            this.dragging = {
                ev: ev.data, anchor: { x: this.tilePosition.x - startPosition.x, y: this.tilePosition.y - startPosition.y }
            }
        })
        .on('mousemove', function () {
            if (this.dragging) {
                const newPosition = this.dragging.ev.getLocalPosition(this.parent);
                this.tilePosition.x = this.dragging.anchor.x + newPosition.x;
                this.tilePosition.y = this.dragging.anchor.y + newPosition.y;
            }
        })
        .on('mouseup', function () { this.dragging = null })
        .on('mouseupoutside', function () { this.dragging = null })
        ;

    pixi.stage.addChild(tile)

    const rect = new Graphics()
        .beginFill(0xff0000)
        .drawRect(0, 0, 100, 100);
    pixi.stage.addChild(rect);

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
    const spawn = document.getElementById('spawn') as HTMLButtonElement
    spawn.addEventListener('click', () => {
        fetch('/api/spawn', { method: 'POST', body: JSON.stringify({ program }), headers: { 'Content-Type': 'application/json' } })
    })
    document.getElementById('compile')!.addEventListener('click', () => {
        const code = document.getElementById('code') as HTMLInputElement
        fetch('/api/compile', { method: 'POST', body: code.files![0] }).then(res => res.json()).then(p => {
            program = p
            spawn.disabled = false
            spawn.textContent = 'Spawn ' + program
        })
    })
})
