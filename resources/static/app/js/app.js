
var connected = false;
var subscriptions = [];

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
    stompClient.subscribe('/markets/0/depth', message => {
        handleDepth(message.body);
    });
    stompClient.subscribe('/markets/0/last_trade', message => {
        handleLastTrade(message.body);
    });
    for (const subscription of subscriptions) {
        sendsubscribe(subscription);
    }
};

function subscribe(accountKey) {
    subscriptions.push(accountKey);
    if (!connected) {
        return;
    }
    sendsubscribe(accountKey);
}

function sendsubscribe(accountKey) {
    console.log('Subscribing to ' + accountKey);
    destination = '/accounts/' + accountKey + '/updates';
    stompClient.subscribe(destination, message => {
        handleUpdate(message.body);
    });
    stompClient.publish({destination: destination, body: JSON.stringify({ request: "GET", scope: "balance"})});
    stompClient.publish({destination: destination, body: JSON.stringify({ request: "GET", scope: "positions"})});
    stompClient.publish({destination: destination, body: JSON.stringify({ request: "GET", scope: "orders"})});

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
    stompClient.activate();
    getAccounts();
}


function handleDepth(message) {
    // console.log('Got depth message: ' + message);
    let depth = JSON.parse(message);

    $("#markets_body").append(
        "<tr>"
        + "<td>"+ depth.instrument_id + "</td>"
        + "<td>" + message + "</td>"
        + "</tr>");
}

function handleLastTrade(message) {
    // console.log('Got last trade message: ' + message);
    let last_trade = JSON.parse(message);

    $("#markets_body").append(
        "<tr>"
        + "<td>"+ last_trade.instrument_id + "</td>"
        + "<td>" + message + "</td>"
        + "</tr>");
}

function handleUpdate(message) {
    // console.log('Got message: ' + message);
    let account_update = JSON.parse(message);
    // console.log('Parsed: ' + account_update);

    balance = account_update.balance;
    position = account_update.position;
    order_state = account_update.order_state;
    //
    // console.log('Got position: ' + position);
    // console.log('Got balance: ' + balance);
    // console.log('Got order_state: ' + order_state);

    if (balance !== undefined && balance !== null) {
        $("#balances_body").append(
            "<tr>"
            + "<td>" + balance.cash + "</td>"
            + "</tr>");
    }
    if (position !== undefined && position !== null) {
        $("#positions_body").append(
            "<tr>"
            + "<td>" + position.instrument_id + "</td>"
            + "<td>" + position.quantity + "</td>"
            + "<td>" + position.cost + "</td>"
            + "<td>" + position.closed_gain + "</td>"
            + "<td>  <button id=\"close\" class=\"btn btn-default\" type=\"submit\">Close</button>\n </td>"
            + "</tr>");
    }
    if (order_state !== undefined && order_state !== null) {
        $("#orders_body").append(
            "<tr>"
            + "<td>"+ order_state.order.order_number + "</td>"
            + "<td>" + order_state.order_status + "</td>"
            + "<td>"+ order_state.order.quantity + "</td>"
            + "<td>"+ order_state.order.price + "</td>"
            + "<td>  <button id=\"cancel\" class=\"btn btn-default\" type=\"submit\">Cancel</button>\n </td>"
            + "</td>");
    }
}

$(function () {
    $("form").on('submit', (e) => e.preventDefault());
    $( "#connect" ).click(() => connect());
});

function processAccounts(account_data) {
    console.log('Got accounts:' + account_data);
    $("#account-data").show();

    for (account of account_data) {
        console.log('account: ' + account);
        $("#accounts_body").append(
            "<tr>"
            + "<td>"+ account.account_number + "</td>"
            + "<td>" + account.privileges + "</td>");
        subscribe(account.account_key);
        break;
    }

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

connect();