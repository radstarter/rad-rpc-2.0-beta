# rad-rpc-2.0
## Usage 
Create a Config.json file either manually or edit the function `setup::create_setup_file_example()`.
Make sure to run it from main if you use the function and to comment it 
out if you manually edit the JSON. This file exists so the system can
recover from a crash quickly and setup all the packages and components
automatically on start.


Once you've set-up Config.json you can just `cargo run` and it will
setup the ledger in a clean state. And import the wasm files you've
specified in the Config. The server will expose port 3030 and
you can make POST requests to it according to the official JSON-RPC-2.0
spec. See the examples dir for how to do this in Js.


Current exposed interfaces:\
1. create_account
2. call_function
3. call_method
4. get_balance


