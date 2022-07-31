import * as PIXI from "pixi.js";
import { Viewport } from "pixi-viewport";
import { FPS } from 'yy-fps';
import * as Honeycomb from "honeycomb-grid";

const INITIAL_RAD = 100
document.addEventListener('DOMContentLoaded', () => {
    const fps = new FPS()
    PIXI.Ticker.shared.add(() => fps.frame())

    const canvas = new PIXI.Application({ resizeTo: document.body })
    document.body.appendChild(canvas.view)

    const Hex = Honeycomb.extendHex({ origin: Honeycomb.extendHex()(0).center() })
    const Grid = Honeycomb.defineGrid(Hex)

    const viewport = new Viewport({ screenWidth: canvas.view.width, screenHeight: canvas.view.height, passiveWheel: true })
    viewport.scale.set(INITIAL_RAD)
    canvas.renderer.on('resize', () => {
        const center = viewport.center.clone()
        viewport.resize(canvas.view.width, canvas.view.height)
        viewport.moveCenter(center)
        viewport.emit('zoomed')
        viewport.emit('moved')
    })

    /*const tileGrid = new PIXI.TilingSprite(PIXI.Texture.from(new URL(
        'data/tile.png',
        import.meta.url
    ).toString(), { width: 127, height: 220 }), viewport.screenWidth, viewport.screenHeight)
    viewport.on('moved', () => {
        tileGrid.width = canvas.view.width
        tileGrid.height = canvas.view.height
        tileGrid.tilePosition.set(-viewport.left * viewport.scale.x, (tileGrid.texture.height/6-viewport.top) * viewport.scale.y)
        tileGrid.tileScale.set(viewport.scale.x / tileGrid.texture.width * 2 * CELL_RAD, viewport.scale.y / tileGrid.texture.height * 2 * CELL_RAD)
    })*/

    const zero = new PIXI.Sprite(PIXI.Texture.WHITE)
    zero.width = .2
    zero.height = .2
    zero.position.set(0, 0)
    const vecx = new PIXI.Sprite(PIXI.Texture.WHITE)
    vecx.width = .2
    vecx.height = .2
    vecx.position.set(1*2, 0)
    vecx.tint = 0xff0000
    const vecy = new PIXI.Sprite(PIXI.Texture.WHITE)
    vecy.width = .2
    vecy.height = .2
    vecy.position.set(1, 1*2)
    vecy.tint = 0x00ff00
    const vecz = new PIXI.Sprite(PIXI.Texture.WHITE)
    vecz.width = .2
    vecz.height = .2
    vecz.position.set(-1, 1*2)
    vecz.tint = 0x0000ff
    viewport.addChild(zero, vecx, vecy, vecz)

    const grid = new PIXI.Graphics()
    viewport.on('moved', () => {
        grid.clear()
        grid.lineStyle({ color: 0xFFFFFF, /*alpha: .5,*/ width: 1, native: true })
        const box = viewport.getVisibleBounds()
        
        // const view = Grid.hexagon({ radius: Math.floor(Math.min(box.width, box.height) / 3) - 1, center: Hex().fromPoint(viewport.center) })
        const siz = Hex().fromPoint(box.width, box.height)
        const view = Grid.rectangle({ width: siz.x+1, height: siz.y+1, start: Hex().fromPoint(viewport.left, viewport.top) })

        view.forEach(hex => {
            const point = hex.toPoint()
            // add the hex's position to each of its corner points
            const corners = hex.corners().map(corner => corner.add(point))
            // separate the first from the other corners
            const [firstCorner, ...otherCorners] = corners

            // move the "pen" to the first corner
            grid.moveTo(firstCorner.x, firstCorner.y)
            // draw lines to the other corners
            otherCorners.forEach(({ x, y }) => grid.lineTo(x, y))
            // finish at the first corner
            grid.lineTo(firstCorner.x, firstCorner.y)
        })
    })
    viewport.addChild(grid)

    canvas.stage.addChild(viewport)

    viewport
        .moveCenter(0, 0)
        .drag({ wheel: true })
        .pinch()
        .wheel()
        //.mouseEdges()
        /*.clampZoom({
            //minScale, maxScale
        })*/
    viewport.emit('moved')

    net()
})

function net() {
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
}