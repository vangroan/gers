import "game" for Game
import "graphics" for GraphicDevice, VertexArrayObject, VertexArray, Vertex, Texture
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

    if (Keyboard.isKeyPressed("W")) {
      System.print("UP")
    }

    if (Keyboard.isKeyPressed("S")) {
      System.print("DOWN")
    }

    if (Keyboard.isKeyPressed("A")) {
      System.print("LEFT")
    }

    if (Keyboard.isKeyPressed("D")) {
      System.print("RIGHT")
    }

    if (Mouse.isButtonPressed("Left")) {
      System.print("MOUSE LEFT")
    }
  }

  draw() {
    // System.print("GraphicDevice %(_device)")
    _device.clearScreen(8, 16, 24, 255)
    // _device.clearScreen(128, 200, 255, 255)
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
    vertex.setColor(1.0, 0.0, 0.0, 1.0)
    vertices.add(vertex)

    vertex.setPos(1.0, 0.0)
    vertex.setUv(1.0, 0.0)
    vertex.setColor(0.0, 1.0, 0.0, 1.0)
    vertices.add(vertex)

    vertex.setPos(1.0, 1.0)
    vertex.setUv(1.0, 1.0)
    vertex.setColor(0.0, 0.0, 1.0, 1.0)
    vertices.add(vertex)

    vertex.setPos(0.0, 1.0)
    vertex.setUv(0.0, 1.0)
    vertex.setColor(0.0, 1.0, 1.0, 1.0)
    vertices.add(vertex)

    System.print("%(vertices.toString())")

    // Vertex Array Object
    System.print("GraphicDevice %(GraphicDevice.instance)")
    var vao = VertexArrayObject.new(GraphicDevice.instance, vertices, indices)

    // Texture
    var texture = Texture.new(GraphicDevice.instance, 512, 512)
  }
}

MyGame.new()

// Game.run(MyGame.new())
// MyGame.testArray()
