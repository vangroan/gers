import "game" for Game
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
  }
}

Game.run(MyGame.new())
