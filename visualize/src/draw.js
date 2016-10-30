export default {
  arrow(ctx, pt, angle, size=10) {
    var alx = pt.x - size*Math.cos(angle - Math.PI/6),
        aly = pt.y - size*Math.sin(angle - Math.PI/6)
    var arx = pt.x - size*Math.cos(angle + Math.PI/6),
        ary = pt.y - size*Math.sin(angle + Math.PI/6)
    var asx = pt.x - size*.7*Math.cos(angle),
        asy = pt.y - size*.7*Math.sin(angle)
    ctx.moveTo(pt.x, pt.y)
    ctx.lineTo(alx, aly)
    ctx.lineTo(asx, asy)
    ctx.lineTo(arx, ary)
    ctx.lineTo(pt.x, pt.y)
  },
  shape (ctx, attrs, x, y, side) {
    const dX = Math.sin(attrs.angle) * side / 2
    const dY = Math.cos(attrs.angle) * side / 2
    const cX = x + side / 2
    const cY = y + side / 2
    const gradient = ctx.createLinearGradient(cX - dX, cY - dY, cX + dX, cY + dY)
    gradient.addColorStop(0, attrs.colors[0])
    gradient.addColorStop(1, attrs.colors[1])
    ctx.save()
    ctx.fillStyle = gradient
    if(attrs.shape === 0) {
      ctx.fillRect(x + side / 4, y + side / 4, side / 2, side / 2)
    } else if(attrs.shape === 1) {
      ctx.beginPath()
      ctx.moveTo(cX, side / 4)
      ctx.lineTo(x + side * 3 / 4, y + side * 6 / 8)
      ctx.lineTo(x + side / 4, y + side * 6 / 8)
      ctx.closePath()
      ctx.fill()
      ctx.stroke()
    } else if(attrs.shape === 2) {
      ctx.beginPath()
      ctx.arc(cX, cY, side / 4, 0, 2 * Math.PI, false)
      ctx.fill()
      ctx.stroke()
    }
    ctx.restore()
    // ctx.fillText(next.value.text, i * side + side / 2, side / 2)
  },

  setup (canvas, ctx, transX, transY) {
    ctx.globalCompositeOperation = 'destination-over'
    // clear canvas
    ctx.clearRect(-transX, -transY, canvas.width, canvas.height)

    // drawing code
    ctx.strokeStyle = "black"
    ctx.fillStyle = "black"
    ctx.font = '12pt Calibri'
    ctx.textAlign = 'center'
  },
}
