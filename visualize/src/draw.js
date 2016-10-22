export function drawArrow(ctx, pt, angle, size=10) {
  var alx = pt.x - size*Math.cos(angle - Math.PI/6),
      aly = pt.y - size*Math.sin(angle - Math.PI/6);
  var arx = pt.x - size*Math.cos(angle + Math.PI/6),
      ary = pt.y - size*Math.sin(angle + Math.PI/6);
  var asx = pt.x - size*.7*Math.cos(angle),
      asy = pt.y - size*.7*Math.sin(angle);
  ctx.moveTo(pt.x, pt.y);
  ctx.lineTo(alx, aly);
  ctx.lineTo(asx, asy);
  ctx.lineTo(arx, ary);
  ctx.lineTo(pt.x, pt.y);
}
