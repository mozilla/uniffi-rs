/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import SwiftUI

struct ContentView: View {
    @State private var clicked = 0
    var body: some View {
        VStack {
            if clicked <= 0 {
                Text("Click me!").padding()
            } else {
                Text("Clicked \(clicked) times!").padding()
            }

            HStack {
                if clicked > 0 {
                    Button("-") {
                        clicked = clicked - 1
                    }.padding()
                } else {
                    Text("-").padding()
                }
                Button("+") {
                    testFunc()
                    clicked = clicked + 1
                }.padding()
            }
        }
    }

    func testFunc() {
        let bobo = TodoList()
        try! bobo.addItem(todo: "test test")
        print(try! bobo.getFirst())
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
