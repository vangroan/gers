import "game" for Game
import "input" for Keyboard

class MyGame is Game {
  construct new() {
    super.setup(this)
  }

  init() {
    Keyboard.subscribeChar {|char|
      System.print("Character subscriber %(char)")
    }
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
  }
}

Game.run(MyGame.new())
