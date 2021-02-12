import "game" for Game
import "input" for KeyboardInput

class MyGame is Game {
  construct new() {
    super.setup(this)
  }

  init() {
    KeyboardInput.subscribeChar {|char|
      System.print("Character subscriber %(char)")
    }
  }

  update() {
    // System.print("update")
    // while (poll()) {
    //   System.print(event())
    // }

    for (event in events) {
      System.print(event)
    }

    if (KeyboardInput.isKeyPressed("W")) {
      System.print("UP")
    }

    if (KeyboardInput.isKeyPressed("S")) {
      System.print("DOWN")
    }

    if (KeyboardInput.isKeyPressed("A")) {
      System.print("LEFT")
    }

    if (KeyboardInput.isKeyPressed("D")) {
      System.print("RIGHT")
    }
  }
}

Game.run(MyGame.new())
