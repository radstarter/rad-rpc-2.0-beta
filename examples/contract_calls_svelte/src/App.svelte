<script>
    import { JSONRPCClient } from "json-rpc-2.0";

    let account = "0";
    let key = "0";
    let accountGum = "0";
    let accountXRD = "0";

    let machine = "0";
    let machineGum = "0";
    let machineXRD = "0";

    let price = "0";
    let xrd = "0";
    let gumballAddr = "0";

    // JSONRPCClient needs to know how to send a JSON-RPC request.
    // Tell it by passing a function to its constructor. The function must take a JSON-RPC request and send it.
    const client = new JSONRPCClient((jsonRPCRequest) =>
        fetch("http://127.0.0.1:3030", {
            method: "POST",
            headers: {
                "content-type": "application/json",
            },
            body: JSON.stringify(jsonRPCRequest),
        }).then((response) => {
            if (response.status === 200) {
                // Use client.receive when you received a JSON-RPC response.
                return response
                    .json()
                    .then((jsonRPCResponse) => client.receive(jsonRPCResponse));
            } else if (jsonRPCRequest.id !== undefined) {
                return Promise.reject(new Error(response.statusText));
            }
        })
    );

    // Use client.request to make a JSON-RPC request call.
    // The function returns a promise of the result.
    //
    // Creates a new account when loading the page
    client
        .request("new_account",)
        .then((result) => { 
            console.log(result.account);
            account = result.account;
            key = result.key;
        })

    async function callFunction() {
        let result = await client.request("call_function",  [
        "01e192213f8ae4d9ae27f6a5fd2ed30df6f1449947c4280b1afac2",
        "GumballMachine",
        "new",
        ["1.3"],
        account,
        key
       ]);
        console.log(result);
        machine = result.components[0];
        gumballAddr = result.resources[0];

        let resultGetBal = await client.request("get_balance", [machine]);
        machineGum = resultGetBal[gumballAddr];
    }

    async function callMethodGetPrice() {
        let result = await client.request("call_method", [
            machine,
            "get_price",
            [],
            account,
            key
        ]);
        console.log(result);
        price = result[0];
    }

    async function callMethodBuyGumball() {
        let result = await client.request("call_method", [
            machine,
            "buy_gumball",
            [xrd.toString() + ",030000000000000000000000000000000000000000000000000004"],
            account,
            key
        ]);
        console.log(result);

        let resultGetBal = await client.request("get_balance", [machine]);
        machineGum = resultGetBal[gumballAddr];
        machineXRD = resultGetBal["030000000000000000000000000000000000000000000000000004"];

    }

    async function callShowBalances(address, token) {
        let result = await client.request("get_balance", [address]);
        console.log(result)
        return result[token];
    }

</script>

<main>
{#if client}
    <h1>Gumball Machine </h1>
    account address: {account}
    <p><button on:click={callFunction}>New Gumball Machine</button></p>
    <p>Machine addres: {machine}</p>
    <p>Amount of Gumballs: {machineGum}</p>
    <p>XRD: {machineXRD} </p>
    <p><button on:click={callMethodGetPrice}>Get price</p>
    Price: {price}
    <p><input type="number" bind:value={xrd}> XRD</p>
    <button on:click={callMethodBuyGumball}>Buy gumballs</button>
{:else}
    Couldn't connect to server!
{/if}
</main>

<style>
</style>
