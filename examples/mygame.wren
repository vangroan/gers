import "game" for Game
import "graphics" for GraphicDevice
import "input" for Keyboard, Mouse
import "collections" for U16Array

class MyGame is Game {
  construct new() {
    // Note: Graphic Device is not ready here
    super.setup(this)

    _timer = 0.0
  }

  init() {

  }

  update() {
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
    GraphicDevice.instance.clearScreen(1, 12, 123, 255)
  }
}

Game.run(MyGame.new())
