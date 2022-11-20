registerPaint('gameboy', class {
  static get inputProperties() { return ['--inner-background', '--animation-tick']; }
  paint(ctx, geom, properties) {
    const rippleColor = properties.get('--inner-background').toString();
    let tick = parseFloat(properties.get('--animation-tick').toString());
    if (tick < 0)
      tick = 0;
    if (tick > 1000)
      tick = 1000;
    const colorArray = rippleColor.split("|");

    let center_x = parseInt(geom.width / 2);
    let center_y = parseInt(geom.height / 2);

    colorArray.forEach((color, index) => {
      ctx.beginPath();
      ctx.fillStyle = color;
      let radius = geom.width * tick / 1000 - index * 20;
      ctx.arc(
        center_x, center_y, // center
        Math.max(radius, 0), // radius
        0, // startAngle
        2 * Math.PI //endAngle
      );
      ctx.fill();
    });
  }
});
