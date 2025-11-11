
var connected = false;
var subscriptions = {};
var instruments = {};
var accounts = {};
var marketData = {};

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

function computeMarket(instrumentKey) {
    let instrumentData = marketData[instrumentKey];
    console.log("Instrument data = " + JSON.stringify(instrumentData));
    if (instrumentData === undefined) {
        return;
    }

    let bid = undefined;
    let bidSize = undefined;
    let ask = undefined;
    let askSize = undefined;
    let last = undefined;

    let depth = instrumentData['depth'];
    if (depth !== undefined) {
        let buys = depth['buys'];
        let sells = depth['sells'];

        if (buys !== undefined) {
            let buys0 = buys['0'];
            if (buys0 !== undefined) {
                bid = buys0['price'];
                bidSize = buys0['quantity'];
            }
        }
        if (sells !== undefined) {
            let sells0 = sells[0];
            if (sells0 !== undefined) {
                ask = sells0['ask'];
                askSize = sells0['quantity'];
            }
        }
    }

    let lastTrade = instrumentData['lastTrade'];
    if (lastTrade !== undefined) {
        last = lastTrade.price;
    }
    let mark = computeMark(bid, ask, last);
    return { bid: bid, bidSize: bidSize, ask: ask, askSize: askSize, last: last, mark: mark };
}

function computeMark(bid, ask, last) {
    if (bid !== undefined && ask !== undefined) {
        return (bid + ask) / 2;
    }
    if (bid !== undefined) {
        return bid;
    }
    if (ask !== undefined) {
        return ask;
    }
    if (last !== undefined) {
        return last;
    }
    return undefined;
}

function updatePosition(accountKey, instrumentKey, mark) {
    console.log("updatePosition accountKey " + accountKey + " and instrumentKey " + instrumentKey);
    let position = accounts[accountKey]['positions'][instrumentKey];
    if (position === undefined) {
        console.log("No position for accountKey " + accountKey + " and instrumentKey " + instrumentKey);
        return;
    }
    let id = computePositionRowId(accountKey, instrumentKey);
    let opengainid = "opengain:" + id;
    let netliqid = "netliq:" + id;

    let netLiqElement = document.getElementById(netliqid);
    let openGainElement = document.getElementById(opengainid);

    let netLiq = '-';
    if (mark !== undefined) {
        netLiq = (position.quantity * mark);
        if (netLiqElement !== undefined && netLiqElement != null) {
            netLiqElement.innerHTML = netLiq.toFixed(2);
        }
        else {
            console.log("No position netliq for: " + netliqid);
        }
        if (openGainElement !== undefined && openGainElement != null) {
            openGainElement.innerHTML = (netLiq - position.cost).toFixed(2)
        } else {
            console.log("No position open gain for: " + opengainid);
        }
    } else {
        netLiqElement.innerHTML = "-";
        openGainElement.innerHTML = "-";

    }

}

function updatePositions(instrumentKey, mark) {
    for (let accountKey in accounts) {
        updatePosition(accountKey, instrumentKey, mark);
    }
}

function updateMarketData(instrumentKey) {

    let instrumentData = marketData[instrumentKey];
    console.log("Instrument data = " + JSON.stringify(instrumentData));
    let market = computeMarket(instrumentKey);

    updatePositions(instrumentKey, market.mark);

    let marketDataId = "marketData:" + instrumentKey;
    deleteRow(marketDataId);
    let description = getInstrumentDescription(instrumentKey);
    let symbol = getInstrumentSymbol(instrumentKey);

    let text = "<tr id=" + marketDataId + ">";
    text += "<td title='" + description + "'>" + symbol + "</td>"

    text += "<td>";
    if (market.bid !== undefined && market.bidSize !== undefined) {
        text += market.bid.toFixed(2) + "@" + market.bidSize;
    } else {
        text += "-";
    }
    text += "</td>";

    text += "<td>";
    if (market.mark !== undefined) {
        text += market.mark.toFixed(2);
    } else {
        text += "-";
    }
    text += "</td>";

    text += "<td>";
    if (market.ask !== undefined && market.askSize !== undefined) {
        text += market.ask.toFixed(2) + "@" + market.askSize;
    } else {
        text += "-";
    }
    text += "</td>";

    text += "<td>";
    if (market.last != undefined) {
        text += market.last.toFixed(2);
    } else {
        text += "-";
    }
    text += "</td>";


    $("#markets_table").append(text);

    // let marketDataElement = document.getElementById(marketDataId);
    // if (marketDataElement !== undefined && marketDataElement != null) {
    //     marketDataElement.innerHTML = market;
    // }
    // else {
    //     deleteRow(id);
    // }
}

function setMarketData(instrumentKey, category, data)    {
    if (marketData[instrumentKey] === undefined) {
        marketData[instrumentKey] = {};
    }
    let oldData = marketData[instrumentKey][category];
    if (oldData !== undefined) {
        // console.log("Old market data version number: " + oldData.version_number +
        //     ", new market data version number: " + data.version_number);
        if (oldData.version_number >= data.version_number) {
            return false;
        }
    } else {
        marketData[instrumentKey][category] = data;
    }
    console.log("Market data = " + JSON.stringify(marketData));
    updateMarketData(instrumentKey);

    return true;
}


function handleDepth(stompMessage) {
    let message = stompMessage.body;
   // console.log('Got depth message: ' + message);
    let depth = JSON.parse(message);
    setMarketData(depth.instrument_key, 'depth', depth);
}

function handleLastTrade(stompMessage) {
    let message = stompMessage.body;

  //  console.log('Got last trade message: ' + message);
    let lastTrade = JSON.parse(message);
    setMarketData(lastTrade.instrument_key, 'lastTrade', lastTrade);
}

function handleBalance(balance) {
    if (balance !== undefined && balance !== null) {
        let account = accounts[balance.account_key];

        deleteRow('posbalance');
        $("#positions_body").append(
            "<tr id=posbalance>"
            + "<td title='" + account.account_name + "'>"+ account.account_number + "</td>"
            + "<td title='cash balance'>-cash-</td>"
            + "<td></td>"
            + "<td></td>"
            + "<td></td>"
            + "<td class='.right-align'>" + (balance.cash).toFixed(2) + "</td>"
            + "<td></td>"
            + "<td></td>"
            + "</tr>");
    }
}

function deleteRow(rowid) {
    var row = document.getElementById(rowid);
    if (row !== null) {
        row.parentNode.removeChild(row);
    }
}

function closePosition(accountKey, instrumentKey) {
    let position = accounts[accountKey]['positions'][instrumentKey];
    document.getElementById("order_quantity").value = -1 * position.quantity;
    costBasis = (position.cost / position.quantity).toFixed(2);
    document.getElementById("order_price").value = costBasis;
    document.getElementById("order_instrument").value = position.instrument_key;
}

function computePositionRowId(accountKey, instrumentKey) {
    return "pos:" + accountKey + ":" + instrumentKey;
}

function handlePosition(position) {
  //  console.log("Received position: " + JSON.stringify(position));
    if (position !== undefined && position !== null) {
        let old_position = accounts[position.account_key]['positions'][position.instrument_key];
        if (old_position !== undefined) {
            // console.log("Old position version number: " + old_position.version_number +
            //     ", new position version number: " + position.version_number);
            if (old_position.version_number >= position.version_number) {
                console.log("Old position version number: " + old_position.version_number +
                    ", new position version number: " + position.version_number +
                    ", skipping update");
                return;
            }
        } else {
            accounts[position.account_key]['positions'][position.instrument_key] = position;
        }
        let id = computePositionRowId(position.account_key, position.instrument_key);
        deleteRow(id);

        let description = getInstrumentDescription(position.instrument_key);
        let symbol = getInstrumentSymbol(position.instrument_key);
        let closeButtonId = "close:" + id;

        let actions = "";
        let costBasis = 0;
        if (position.quantity !== 0) {
            costBasis = (position.cost / position.quantity).toFixed(2);
            actions += "<button id='" + closeButtonId + "' class=\"btn btn-default\" type=\"submit\">Close</button>";
        }

        let account = accounts[position.account_key];

        let position_body =
            "<tr id=" + id + ">"
            + "<td title='" + account.account_name + "'>" + account.account_number + "</td>"
            + "<td title='" + description + "'>" + symbol + "</td>"
            + "<td class='right-align'>" + position.quantity + "</td>"
            + "<td class='right-align'>" + costBasis + "</td>"
            + "<td class='right-align'>" + position.cost.toFixed(2) + "</td>"
            + "<td id='netliq:" + id + "' class='right-align'>" + "0" + "</td>"
            + "<td id='opengain:" + id + "' class='right-align'>" + "-" + "</td>"
            + "<td class='right-align'>" + position.closed_gain.toFixed(2) + "</td>"
            + "<td class='right-align'>" + actions + "</td>"
            + "</tr id=" + position.instrument_key + ">";

        console.log(position_body);

        $("#positions_body").append(position_body);
        if (position.quantity !== 0) {
            document.getElementById(closeButtonId).addEventListener("click", () => {
                closePosition(position.account_key, position.instrument_key);
            });
        }
        let market = computeMarket(position.instrument_key);
        if (market !== undefined) {
            updatePosition(position.account_key, position.instrument_key, market.mark);
        }
    }
}

function cancelOrder(accountKey, extOrderId) {
    let xhttp = new XMLHttpRequest();

    let path = "/accounts/" + accountKey + "/orders/" + extOrderId;
    xhttp.open("DELETE", path, true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send();
}

function handleOrderState(orderState) {
    if (orderState !== undefined && orderState !== null) {
        let oldOrderState = accounts[orderState.order.account_key]['orders'][orderState.order.ext_order_id];

        if (oldOrderState !== undefined) {
            // console.log("Old order state version number: " + oldOrderState.version_number +
            //     ", new order state  version number: " + orderState.version_number);
            if (oldOrderState.version_number >= orderState.version_number) {
                console.log("Old order state version number: " + oldOrderState.version_number +
                    ", new order state version number: " + orderState.version_number +
                    ", skipping update");
                return;
            }
        } else {
            accounts[orderState.order.account_key]['orders'][orderState.order.ext_order_id] = orderState;
        }

        let id = "ord:" + orderState.order.account_key + ":" + orderState.order.ext_order_id;
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

        let cancelButtonId = "cancel:" + id;

        let actions = "";
        if (orderState.order_status === 'Pending' ||
            orderState.order_status === 'Open') {
            actions += "<button id='" + cancelButtonId + "' className='btn btn-default' type='submit'>Cancel</button>";
        }
        let account = accounts[orderState.order.account_key];

        let orderStatus = "<td";
        if (orderState.order_status === "Rejected" && orderState.reject_reason !== null) {
            orderStatus += " title='" + orderState.reject_reason + "'";
        }
        orderStatus += " >" + orderState.order_status + "</td>";

        $("#orders_body").append(
            "<tr id=" + id + ">"
            + "<td title='" + account.account_name + "' class='right-align'>"+ account.account_number + "</td>"
            + "<td class='right-align'>"+ orderState.order.order_number + "</td>"
            + "<td title='" + description + "' class='right-align'>" + symbol + "</td>"
            + orderStatus
            + "<td class='right-align'>"+ orderState.order.quantity + "</td>"
            + "<td class='right-align'>"+ orderState.order.price.toFixed(2) + "</td>"
            + "<td>" + actions + "</td>"
            + "</td>");

        if (orderState.order_status === 'Pending' ||
            orderState.order_status === 'Open') {
            document.getElementById(cancelButtonId).addEventListener("click", () => {
                cancelOrder(orderState.order.account_key, orderState.order.ext_order_id);
            });
        }
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
  //  console.log('Got accounts:' + JSON.stringify(account_data));
    accounts = account_data;
    $("#account-data").show();
    let order_account_select = document.getElementById('order_account');

    Object.values(account_data).forEach((account) => {
        //    console.log('account: ' + JSON.stringify(account));
        account['positions'] = {};
        account['orders'] = {};
        $("#accounts_body").append(
            "<tr>"
            + "<td>"+ account.account_number + "</td>"
            + "<td>" + account.account_name + "</td>"
            + "<td>" + account.nickname + "</td>");
        subscribeAccount(account.account_key, handleUpdate);
        var opt = document.createElement('option');
        opt.value = account.account_key;
        opt.innerHTML = account.account_number;
        order_account_select.appendChild(opt);
    });
}

function processInstruments(instrument_data) {
  //  console.log('Got instruments:' + JSON.stringify(instrument_data));
    instruments = instrument_data;
    let order_instrument_select = document.getElementById('order_instrument');

    Object.values(instruments).forEach((instrument) => {
        if (instrument.status != 'Active') {
            return;
        }
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
                ratio: Number(1),
                instrument_key: instrument_key
            }
        ]
    });

    xhttp.send(body);
    document.getElementById("order_quantity").value = "";
    document.getElementById("order_price").value = "";

}

$(function () {
    $("form").on('submit', (e) => e.preventDefault());
    $( "#order_submit" ).click(() => submitOrder());
});

connect();