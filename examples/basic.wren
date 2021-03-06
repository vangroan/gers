import "examples.foobar" for Foobar
import "game" for Game
import "graphics" for GraphicDevice, VertexArrayObject, VertexArray,
  Vertex, Texture, Shader, Transform2D, SpriteBatch, Sprite
import "input" for Keyboard, Mouse
import "collections" for U16Array, U8Array, I8Array

class MyGame is Game {
  construct new() {
    // Note: Graphic Device is not ready here
    super()

    _timer = 0.0
  }

  init() {
    _device = GraphicDevice.instance

    Game.onError.add {|error|
      System.print("Script handled error %(error)")
    }

    System.print("MyGame.init()")
    System.print("GraphicDevice: %(GraphicDevice.instance)")
    // createVertexArray_()
    
    Keyboard.subscribeChar {|char|
      System.print("Character subscriber %(char)")
    }

    Mouse.onButton {|button, state|
      System.print("Mouse state %(button) %(state)")
    }

    MyGame.testArray()
  }

  update() {
    _timer = _timer + Game.deltaTime
    if (_timer > 1.0) {
      System.print("%(Mouse.logicalX), %(Mouse.logicalY), %(Mouse.physicalX), %(Mouse.physicalY)")
      _timer = 0.0
    }

    var dt = Game.deltaTime
    var speed = 100
    var dx = 0.0
    var dy = 0.0

    if (Keyboard.isKeyPressed("W")) {
      dy = dy - 1
    }

    if (Keyboard.isKeyPressed("S")) {
      dy = dy + 1
    }

    if (Keyboard.isKeyPressed("A")) {
      dx = dx - 1
    }

    if (Keyboard.isKeyPressed("D")) {
      dx = dx + 1
    }

    __transform.translate(dx * speed * dt, dy * speed * dt)

    if (Keyboard.isKeyPressed("E")) {
      __transform.rotate(speed * dt)
    }

    if (Keyboard.isKeyPressed("Q")) {
      __transform.rotate(-speed * dt)
    }

    if (Mouse.isButtonPressed("Left")) {
      System.print("MOUSE LEFT")
    }

    for (sprite in __sprites) {
      sprite.transform.rotate(dt * 10.0)
    }
  }

  draw() {
    // System.print("GraphicDevice %(_device)")
    _device.clearScreen(8, 16, 24, 255)
    // _device.clearScreen(128, 200, 255, 255)
    _device.draw(__vao, __texture, Shader.default, __transform)

    for (sprite in __sprites) {
      __batch.add(sprite)
    }
    
    __batch.draw(GraphicDevice.instance, Shader.default, __batch_transform)
  }

  createVertexArray_() {
    // Triangle Indices
    var indices = U16Array.new()
    
    // Triangle 1
    indices.add(0)
    indices.add(1)
    indices.add(2)

    // Triangle 2
    indices.add(0)
    indices.add(2)
    indices.add(3)

    for (item in indices) {
      System.print(item)
    }

    // var vertexArray = VertexArray.new(_device, indices)
    var v = VertexArrayObject.new(GraphicDevice.instance, null, indices)    
  }

  static testArray() {
    // Just some cheeky tests
    var bytes = U8Array.new()
    bytes.add(1)
    bytes.add(2)
    bytes.add(3)
    bytes.add(11111) // overflow
    for (byte in bytes) {
      System.print("Byte: %(byte)")
    }
    
    bytes
      .map {|byte| 2.pow(byte)}
      .each {|byte| System.print("Byte: %(byte)")}

    // i8
    var ints = I8Array.new()
    ints.add(-1)
    ints.add(-1000)
    ints.each {|i| System.print("i8: %(i)") }

    // index
    var indices = U16Array.new()
    indices.add(0)
    indices.add(1)
    indices.add(2)

    indices.add(0)
    indices.add(2)
    indices.add(3)

    // vertex
    // Note: Vertex is copied into array, so we can
    //       reuse the same instance.
    var vertices = VertexArray.new()
    var vertex = Vertex.new()

    vertex.setPos(0.0, 0.0)
    vertex.setUv(0.0, 0.0)
    vertex.setColor(1.0, 1.0, 1.0, 1.0)
    vertices.add(vertex)

    vertex.setPos(200.0, 0.0)
    vertex.setUv(1.0, 0.0)
    vertex.setColor(1.0, 1.0, 1.0, 1.0)
    vertices.add(vertex)

    vertex.setPos(200.0, 200.0)
    vertex.setUv(1.0, 1.0)
    vertex.setColor(1.0, 1.0, 1.0, 1.0)
    vertices.add(vertex)

    vertex.setPos(0.0, 200.0)
    vertex.setUv(0.0, 1.0)
    vertex.setColor(1.0, 1.0, 1.0, 1.0)
    vertices.add(vertex)

    System.print("%(vertices.toString())")

    // Vertex Array Object
    System.print("GraphicDevice %(GraphicDevice.instance)")
    __vao = VertexArrayObject.new(GraphicDevice.instance, vertices, indices)

    // Texture
    // __texture = Texture.new(GraphicDevice.instance, 512, 512)
    __texture = Texture.fromFile(GraphicDevice.instance, "examples/test_pattern.png")

    // Transform
    __transform = Transform2D.new()
    __transform.setPos(100, 100)
    __transform.setOffset(50, 50)

    // Sprite Batch
    __batch = SpriteBatch.new(GraphicDevice.instance)

    // Sprite
    __batch_transform = Transform2D.new()
    // __batch_transform.setPos(110, 10)
    // __batch_transform.setOffset(0, 0)
    // __batch_transform.setScale(2, 2)
    // __batch_transform.rotate(0)

    __sprites = []
    for (y in 0...2) {
      for (x in 0...2) {
        var sprite = Sprite.new()
        sprite.width = 50
        sprite.height = 50
        sprite.texture = __texture
        sprite.transform.rotate(0)
        sprite.transform.setOffset(25, 25)
        sprite.transform.setPos(10 + x * 100, 10 + y * 100)

        __sprites.add(sprite)
      }
    }
  }
}

MyGame.new()

// Game.run(MyGame.new())
// MyGame.testArray()
