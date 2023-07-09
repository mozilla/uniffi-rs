//
//  ContentView.swift
//  iOS-UniFFI-App
//
//  Created by Chen on 2023/7/9.
//

import SwiftUI
import iOS_UniFFI_Framework

struct ContentView: View {
    init(todoList: TodoList) {
        self.todoList = todoList
    }

    var todoList: TodoList
    let stride: UInt64 = UInt64(1)
    @State private var clicked = UInt64.zero
    @State private var text = ""
    var body: some View {
        VStack {
            HStack {
                Text("\(clicked)").padding()
                TextField("New task", text: $text, onCommit:  {
                    try! self.todoList.addEntry(entry: TodoEntry(text: "\(clicked) \(text)"))
                    text = ""
                    clicked = try! add(a: clicked,  b: stride)
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
                            clicked = try! sub(a: clicked,  b: stride)
                        }
                    }
                }
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView(todoList: TodoList())
    }
}
