/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import SwiftUI

struct ContentView: View {
    @State private var clicked = 0
    var body: some View {
        VStack {
            if clicked > 0 {
                Text("Clicked \(clicked) times!").padding()
            }
            Button("Click me") {
                clicked = clicked + 1
            }.padding()
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
