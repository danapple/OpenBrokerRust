
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

function getInstrumentDescription(instrument_key) {
    var instrument = instruments[instrument_key];
    if (!instrument) {
        return "Id: " + instrument_key;
    }
  //  console.log("Instrument description for display" + instrument.description);
    return instrument.symbol + "(" + instrument.description + ")";
}

function handleDepth(stompMessage) {
    let message = stompMessage.body;
   // console.log('Got depth message: ' + message);
    let depth = JSON.parse(message);
    let description = getInstrumentDescription(depth.instrument_key);
    $("#markets_body").append(
        "<tr>"
        + "<td>" + description + "</td>"
        + "<td>" + message + "</td>"
        + "</tr>");
}

function handleLastTrade(stompMessage) {
    let message = stompMessage.body;

  //  console.log('Got last trade message: ' + message);
    let last_trade = JSON.parse(message);
    let description = getInstrumentDescription(last_trade.instrument_key);

    $("#markets_body").append(
        "<tr>"
        + "<td>"+ description + "</td>"
        + "<td>" + message + "</td>"
        + "</tr>");
}

function handleBalance(balance) {
    if (balance !== undefined && balance !== null) {
        $("#balances_body").append(
            "<tr>"
            + "<td>" + balance.cash + "</td>"
            + "</tr>");
    }
}

function handlePosition(position) {
    if (position !== undefined && position !== null) {
        $("#positions_body").append(
            "<tr>"
            + "<td>" + getInstrumentDescription(position.instrument_key) + "</td>"
            + "<td>" + position.quantity + "</td>"
            + "<td>" + position.cost + "</td>"
            + "<td>" + position.closed_gain + "</td>"
            + "<td>  <button id=\"close\" class=\"btn btn-default\" type=\"submit\">Close</button>\n </td>"
            + "</tr>");
    }
}

function handleOrderState(orderState) {
    if (orderState !== undefined && orderState !== null) {
        let description = "";
        for (const [_, leg] of Object.entries(orderState.order.legs)) {
            if (description.length > 0) {
                description += "/";
            }
            description += getInstrumentDescription(leg.instrument_key);
        }

        $("#orders_body").append(
            "<tr>"
            + "<td>"+ orderState.order.order_number + "</td>"
            + "<td>" + description + "</td>"
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

    for (account of account_data) {
    //    console.log('account: ' + JSON.stringify(account));
        $("#accounts_body").append(
            "<tr>"
            + "<td>"+ account.account_number + "</td>"
            + "<td>" + account.privileges + "</td>");
        subscribeAccount(account.account_key, handleUpdate);
        break;
    }
}

function processInstruments(instrument_data) {
 //   console.log('Got instruments:' + JSON.stringify(instrument_data));
    instruments = instrument_data;

    Object.values(instruments).forEach((instrument) => {
  //      console.log('instrument: ' + JSON.stringify(instrument));
        subscribe('/markets/' + instrument.instrument_key + '/depth', handleDepth);
        subscribe('/markets/' + instrument.instrument_key + '/last_trade', handleLastTrade);
    })
    getAccounts();
    stompClient.activate();
}

function getAccounts() {
    var xhttp = new XMLHttpRequest();
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
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            processInstruments(JSON.parse(this.responseText));
        }
    };
    xhttp.open("GET", "/instruments", true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send();
}

connect();