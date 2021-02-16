import "game" for Game
import "graphics" for GraphicDevice, VertexBuffer
import "input" for Keyboard, Mouse
import "collections" for U16Array

class MyGame is Game {
  construct new() {
    // Note: Graphic Device is not ready here
    super.setup(this)

    _timer = 0.0
  }

  init() {
    _device = GraphicDevice.instance

    System.print("MyGame.init()")
    createVertexArray_()
    
    Keyboard.subscribeChar {|char|
      System.print("Character subscriber %(char)")
    }

    Mouse.onButton {|button, state|
      System.print("Mouse state %(button) %(state)")
    }
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
    _device.clearScreen(4, 8, 12, 255)
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
    var v = VertexBuffer.new(GraphicDevice.instance, null, indices)
  }
}

Game.run(MyGame.new())
