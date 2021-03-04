
foreign class SpriteBatch {
  foreign static new(device)

  add(sprite) {
    add_(sprite.x, sprite.y, sprite.width, sprite.height, sprite.texture, sprite.transform)
  }

  foreign add_(x, y, width, height, texture, transform)
  foreign draw(device, shader)
  foreign draw(device, shader, transform)
}
