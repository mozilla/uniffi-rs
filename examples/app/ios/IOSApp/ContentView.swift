/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import SwiftUI

struct ContentView: View {
    init(todoList: TodoList) {
        self.todoList = todoList
    }

    var todoList: TodoList
    @State private var clicked = 0
    @State private var text = ""
    var body: some View {
        VStack {
            HStack {
                Text("\(clicked)").padding()
                TextField("New task", text: $text, onCommit:  {
                    try! self.todoList.addEntry(entry: TodoEntry(text: "\(clicked) \(text)"))
                    text = ""
                    clicked = clicked + 1
                }).padding()
            }

            List{
                Section(header: Text("Still To Do")) {
                    ForEach(todoList.getEntries(), id: \.self) { entry in
                        Text(entry.text)
                    }
                    .onDelete { index in
                        if let i = index.first {
                            let entry = todoList.getEntries()[i]
                            try! todoList.clearItem(todo: entry.text)
                            clicked = clicked - 1
                        }
                    }
                }
            }
        }
    }
}

extension TodoEntry: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(text)
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView(todoList: TodoList())
    }
}
