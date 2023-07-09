//
//  iOS_UniFFI_App.swift
//  iOS-UniFFI-App
//
//  Created by Chen on 2023/7/9.
//

import SwiftUI
import iOS_UniFFI_Framework

@main
struct iOS_UniFFI_App: App {
    var body: some Scene {
        WindowGroup {
            ContentView(todoList: TodoList())
        }
    }
}
