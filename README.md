# sbx-tool

Dll hack for Twinkle ☆ Crusaders -Starlit Brave Xtream!!-  (version [1.19](https://lillian.jp/support/support.html))  
[vndb](https://vndb.org/v5937)  
[official](https://lillian.jp/kurukuru2/sbx.html)

Still work in progress.
Written in [Rust](https://www.rust-lang.org/) language.

# Special Thanks  
Thank you for the high quality libraries.  
[imgui](https://github.com/ocornut/imgui) by [ocornut](https://github.com/ocornut) and its [Rust binding](https://github.com/imgui-rs/imgui-rs)  
[ilhook-rs](https://github.com/regomne/ilhook-rs) by [regomne](https://github.com/regomne)  
[imgui-impl-win32-rs](https://github.com/super-continent/imgui-impl-win32-rs) by [super-continent](https://github.com/super-continent)  

# Build
TODO

### Debug Console
Build the project with debug profile to enable debug console.  
![](dbg_console.png)  

# TODOs  
- [ ] Reverse thread messages(main loop hook is already done, need to figure out about message it self)
- [ ] Reverse wndproc
- [ ] Figure out about 'character context'(where the client holds character informations such as a frame position.) [wip](https://github.com/d42ejh/sbx-tool/blob/450761f4b083f480ac790682bb5e311587863615/sbx-tool-core/src/battle/mod.rs#L50) Need more debug!

# 日本人
https://github.com/d42ejh/sbx-tool-dll  
居ないと思いますが機能追加も歓迎です!  
なにか質問があればissuesかメールにおねがいします。
