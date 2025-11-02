
var connected = false;
var subscriptions = {};
var instruments = {};

const stompClient = new StompJs.Client({
    webSocketFactory: function () {
        return new WebSocket("/ws");
    }
});

stompClient.onopen = function() {
    console.log("Stomp Client opened");
}

stompClient.onConnect = (frame) => {
    setConnected();
    console.log('Connected: ' + frame);
    for (const [destination, func] of Object.entries(subscriptions)) {
        sendsubscribe(destination, func);
    }
};

function subscribe(destination, func) {
  //  console.log('Subscribing: ' + destination);
    subscriptions[destination] = func;
    if (!connected) {
        return;
    }
    sendsubscribe(destination, func);
}

function subscribeAccount(accountKey, func) {
    let destination = '/accounts/' + accountKey + '/updates';
    subscribe(destination, func);
}

function sendsubscribe(destination, func) {
  //  console.log('sendsubscribe to ' + destination);
    stompClient.subscribe(destination, func);
    if (destination.startsWith('/accounts/')) {
        stompClient.publish({destination: destination, body: JSON.stringify({request: "GET", scope: "balance"})});
        stompClient.publish({destination: destination, body: JSON.stringify({request: "GET", scope: "positions"})});
        stompClient.publish({destination: destination, body: JSON.stringify({request: "GET", scope: "orders"})});
    }
}

stompClient.onWebSocketError = (error) => {
    console.error('Error with websocket', error);
    connected = false;
};

stompClient.onStompError = (frame) => {
    console.error('Broker reported error: ' + frame.headers['message']);
    console.error('Additional details: ' + frame.body);
};

function setConnected() {
    connected = true;
}

function connect() {
    console.log('Connecting...')
    // let cookie = 'api_key=' + $("#apiKeyElement").val();
    // document.cookie = cookie;
    getInstruments();
}

function getInstrumentSymbol(instrument_key) {
    var instrument = instruments[instrument_key];
    if (!instrument) {
        return "Id: " + instrument_key;
    }
    return instrument.symbol;
}

function getInstrumentDescription(instrument_key) {
    var instrument = instruments[instrument_key];
    if (!instrument) {
        return "Id: " + instrument_key;
    }
    return instrument.description;
}

function handleDepth(stompMessage) {
    let message = stompMessage.body;
   // console.log('Got depth message: ' + message);
    let depth = JSON.parse(message);
    let description = getInstrumentDescription(depth.instrument_key);
    let symbol = getInstrumentSymbol(depth.instrument_key);

    $("#markets_body").append(
        "<tr>"
        + "<td title='" + description + "'>" + symbol + "</td>"
        + "<td>" + message + "</td>"
        + "</tr>");
}

function handleLastTrade(stompMessage) {
    let message = stompMessage.body;

  //  console.log('Got last trade message: ' + message);
    let last_trade = JSON.parse(message);
    let description = getInstrumentDescription(last_trade.instrument_key);
    let symbol = getInstrumentSymbol(last_trade.instrument_key);

    $("#markets_body").append(
        "<tr>"
        + "<td title='" + description + "'>" + symbol + "</td>"
        + "<td>" + message + "</td>"
        + "</tr>");
}

function handleBalance(balance) {
    if (balance !== undefined && balance !== null) {
        deleteRow('posbalance');
        $("#positions_body").append(
            "<tr id=posbalance>"
            + "<td title='cash balance'>-cash-</td>"
            + "<td></td>"
            + "<td></td>"
            + "<td>" + (balance.cash).toFixed(2) + "</td>"
            + "<td></td>"
            + "<td></td>"
            + "</tr>");
    }
}

function deleteRow(rowid)
{
    var row = document.getElementById(rowid);
    if (row !== null) {
        row.parentNode.removeChild(row);
    }
}

function handlePosition(position) {
    if (position !== undefined && position !== null) {
        let id = "pos:" + position.instrument_key;
        deleteRow(id);

        let description = getInstrumentDescription(position.instrument_key);
        let symbol = getInstrumentSymbol(position.instrument_key);

        $("#positions_body").append(
            "<tr id=" + id + ">"
            + "<td title='" + description + "'>" + symbol + "</td>"
            + "<td>" + position.quantity + "</td>"
            + "<td>" + position.cost + "</td>"
            + "<td>" + (position.cost * position.quantity).toFixed(2) + "</td>"
            + "<td>" + position.closed_gain + "</td>"
            + "<td>  <button id=\"close\" class=\"btn btn-default\" type=\"submit\">Close</button>\n </td>"
            + "</tr id=" + position.instrument_key + ">");
    }
}

function handleOrderState(orderState) {
    if (orderState !== undefined && orderState !== null) {
        let id = "ord:" + orderState.order.ext_order_id;
        deleteRow(id);

        let symbol = "";
        let description = "";
        for (const [_, leg] of Object.entries(orderState.order.legs)) {
            if (description.length > 0) {
                description += "/";
            }
            description += getInstrumentDescription(leg.instrument_key);
            if (symbol.length > 0) {
                symbol += "/";
            }
            symbol += getInstrumentSymbol(leg.instrument_key);
        }

        $("#orders_body").append(
            "<tr id=" + id + ">"
            + "<td>"+ orderState.order.order_number + "</td>"
            + "<td title='" + description + "'>" + symbol + "</td>"
            + "<td>" + orderState.order_status + "</td>"
            + "<td>"+ orderState.order.quantity + "</td>"
            + "<td>"+ orderState.order.price + "</td>"
            + "<td>  <button id=\"cancel\" class=\"btn btn-default\" type=\"submit\">Cancel</button>\n </td>"
            + "</td>");
    }
}
function handleUpdate(stompMessage) {
    let message = stompMessage.body;
    let account_update = JSON.parse(message);

    balance = account_update.balance;
    position = account_update.position;
    orderState = account_update.order_state;
    handleBalance(balance);
    handlePosition(position);
    handleOrderState(orderState);
}

function processAccounts(account_data) {
   // console.log('Got accounts:' + JSON.stringify(account_data));
    $("#account-data").show();
    let order_account_select = document.getElementById('order_account');

    for (account of account_data) {
    //    console.log('account: ' + JSON.stringify(account));
        $("#accounts_body").append(
            "<tr>"
            + "<td>"+ account.account_number + "</td>"
            + "<td>" + account.account_name + "</td>");
        subscribeAccount(account.account_key, handleUpdate);
        var opt = document.createElement('option');
        opt.value = account.account_key;
        opt.innerHTML = account.account_number;
        order_account_select.appendChild(opt);
        break;
    }
}

function processInstruments(instrument_data) {
 //   console.log('Got instruments:' + JSON.stringify(instrument_data));
    instruments = instrument_data;
    let order_instrument_select = document.getElementById('order_instrument');

    Object.values(instruments).forEach((instrument) => {
  //      console.log('instrument: ' + JSON.stringify(instrument));
        subscribe('/markets/' + instrument.instrument_key + '/depth', handleDepth);
        subscribe('/markets/' + instrument.instrument_key + '/last_trade', handleLastTrade);
        var opt = document.createElement('option');
        opt.value = instrument.instrument_key;
        opt.innerHTML = instrument.description;
        order_instrument_select.appendChild(opt);
    })

    getAccounts();

    stompClient.activate();
}

function getAccounts() {
    let xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            processAccounts(JSON.parse(this.responseText));
        }
    };
    xhttp.open("GET", "/accounts", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send();
}

function getInstruments() {
    let xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            processInstruments(JSON.parse(this.responseText));
        }
    };
    xhttp.open("GET", "/instruments", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send();
}

function submitOrder() {
    let quantity = document.getElementById("order_quantity").value;
    let price = document.getElementById("order_price").value;
    let instrument_key = document.getElementById("order_instrument").value;
    let account_key = document.getElementById("order_account").value;

    // alert("Place order for " + quantity + " @ " + price + " for " + instrument_key);

    let xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            //processInstruments(JSON.parse(this.responseText));
        }
    };
    let path = "/accounts/" + account_key + "/orders";
    xhttp.open("POST", path, true);
    xhttp.setRequestHeader("Content-type", "application/json");

    body = JSON.stringify({
        price: Number(price),
        quantity: Number(quantity),
        legs: [
            {
                ratio: Numbergit statu(1),
                instrument_key: instrument_key
            }
        ]
    });
    xhttp.send(body);

}

$(function () {
    $("form").on('submit', (e) => e.preventDefault());
    $( "#order_submit" ).click(() => submitOrder());
});

connect();