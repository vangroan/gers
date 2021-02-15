import "game" for Game
import "graphics" for GraphicDevice
import "input" for Keyboard, Mouse

class MyGame is Game {
  construct new() {
    super.setup(this)

    _timer = 0.0
  }

  init() {
    Keyboard.subscribeChar {|char|
      System.print("Character subscriber %(char)")
    }

    Mouse.onButton {|button, state|
      System.print("Mouse state %(button) %(state)")
    }

    _device = GraphicDevice.instance
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
  }
}

Game.run(MyGame.new())
