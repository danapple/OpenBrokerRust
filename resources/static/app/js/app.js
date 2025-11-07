import {OpenBroker} from "./openbroker.js";

let openBroker = new OpenBroker();

function instrumentCallback(instrument) {
    // console.log("instrumentCallback: " + JSON.stringify(instrument));
    let order_instrument_select = document.getElementById('order_instrument');

    let opt = document.createElement('option');
    opt.value = instrument.instrument_key;
    opt.innerHTML = instrument.description;
    order_instrument_select.appendChild(opt);
}

function accountCallback(account) {
    // console.log("accountCallback: " + JSON.stringify(account));
    $("#accounts_body").append(
        "<td>"+ account.account_number + "</td>"
        + "<td>" + account.account_name + "</td>"
        + "<td>" + account.nickname + "</td>");
    $("#account-data").show();

    let order_account_select = document.getElementById('order_account');

    let opt = document.createElement('option');
    opt.value = account.account_key;
    opt.innerHTML = account.account_number;
    order_account_select.appendChild(opt);
}

function positionCallback(position) {
    // console.log("positionCallback: " + JSON.stringify(position));
    let description = openBroker.getInstrumentDescription(position.instrument_key);
    let symbol = openBroker.getInstrumentSymbol(position.instrument_key);

    let id = computePositionRowId(position.account_key, position.instrument_key);
    let closeButtonId = "close:" + id;

    let actions = "";
    if (position.quantity !== 0) {
        actions += "<button id='" + closeButtonId + "' class=\"btn btn-default\" type=\"submit\">Close</button>";
    }

    let account = openBroker.getAccount(position.account_key);
    deleteRow(id);
    let position_body =
        "<tr id=" + id + ">"
        + "<td title='" + account.account_name + "'>" + account.account_number + "</td>"
        + "<td title='" + description + "'>" + symbol + "</td>"
        + "<td class='right-align'>" + position.quantity + "</td>"
        + "<td class='right-align'>" + render(position.cost_basis) + "</td>"
        + "<td class='right-align'>" + render(position.cost) + "</td>"
        + "<td id='netliq:" + id + "' class='right-align'>" + render(position.net_liq) + "</td>"
        + "<td id='opengain:" + id + "' class='right-align'>" + render(position.open_gain) + "</td>"
        + "<td class='right-align'>" + render(position.closed_gain) + "</td>"
        + "<td class='right-align'>" + actions + "</td>"
        + "</tr>";

    // console.log(position_body);

    $("#positions_body").append(position_body);
    if (position.quantity !== 0) {
        document.getElementById(closeButtonId).addEventListener("click", () => {
            closePosition(position.account_key, position.instrument_key);
        });
    }
}

function orderStateCallback(orderState) {
    // console.log("orderStateCallback: " + JSON.stringify(orderState));
    let id = "ord:" + orderState.order.account_key + ":" + orderState.order.ext_order_id;
    deleteRow(id);

    let symbol = "";
    let description = "";
    for (const [_, leg] of Object.entries(orderState.order.legs)) {
        if (description.length > 0) {
            description += "/";
        }
        description += openBroker.getInstrumentDescription(leg.instrument_key);
        if (symbol.length > 0) {
            symbol += "/";
        }
        symbol += openBroker.getInstrumentSymbol(leg.instrument_key);
    }

    let cancelButtonId = "cancel:" + id;

    let actions = "";
    if (orderState.order_status === 'Pending' ||
        orderState.order_status === 'Open') {
        actions += "<button id='" + cancelButtonId + "' className='btn btn-default' type='submit'>Cancel</button>";
    }
    let account = openBroker.getAccount(orderState.order.account_key);

    let orderStatus = "<td";
    if (orderState.order_status === "Rejected" && orderState.reject_reason !== null) {
        orderStatus += " title='" + orderState.reject_reason + "'";
    }
    orderStatus += " >" + renderOrderStatus(orderState.order_status) + "</td>";

    $("#orders_body").append(
        "<tr id=" + id + ">"
        + "<td title='" + account.account_name + "' class='right-align'>"+ account.account_number + "</td>"
        + "<td class='right-align'>"+ orderState.order.order_number + "</td>"
        + "<td title='" + description + "' class='right-align'>" + symbol + "</td>"
        + orderStatus
        + "<td class='right-align'>"+ orderState.order.quantity + "</td>"
        + "<td class='right-align'>"+ render(orderState.order.price) + "</td>"
        + "<td>" + actions + "</td>"
        + "</td>");

    if (orderState.order_status === 'Pending' ||
        orderState.order_status === 'Open') {
        document.getElementById(cancelButtonId).addEventListener("click", () => {
            openBroker.cancelOrder(orderState.order.account_key, orderState.order.ext_order_id);
        });
    }

}

function balanceCallback(balance, totals) {
    // console.log("balanceCallback: " + JSON.stringify(balance)
    //     + ", totals: " + JSON.stringify(totals));

    let account = openBroker.getAccount(balance.account_key);
    let posbalanceid = "accbal:" + balance.account_key;
    let postotalsid = "acctot:" + balance.account_key;

    deleteRow(posbalanceid);
    deleteRow(postotalsid);

    $("#positions_body").append(
        "<tr id='" + posbalanceid + "'>"
        + "<td title='" + account.account_name + "'>"+ account.account_number + "</td>"
        + "<td title='cash balance'>-cash-</td>"
        + "<td></td>"
        + "<td></td>"
        + "<td></td>"
        + "<td class='.right-align'>" + render(balance.cash) + "</td>"
        + "<td></td>"
        + "<td></td>"
        + "</tr>");
    // TODO maybe don't show sub-totals if there is only one account shown.
    $("#positions_body").append(
        "<tr id='" + postotalsid + "'>"
        + "<td title='" + account.account_name + "'>"+ account.account_number + "</td>"
        + "<td title='sub-totals'>-sub-totals-</td>"
        + "<td></td>"
        + "<td></td>"
        + "<td></td>"
        + "<td class='.right-align'>" + render(totals.net_liq) + "</td>"
        + "<td></td>"
        + "<td></td>"
        + "</tr>");

}

function totalsCallback(totals) {
    // console.log("totalsCallback: " + JSON.stringify(totals));

    let id = "totals:all";
    deleteRow(id);
    $("#positions_body").append(
        "<tr id='" + id + "'>"
        + "<td>All</td>"
        + "<td title='totals'>-totals-</td>"
        + "<td class='right-align'></td>"
        + "<td class='right-align'></td>"
        + "<td class='right-align'>" + render(totals.cost) + "</td>"
        + "<td id='netliq:" + id + "' class='right-align'>" + render(totals.net_liq) + "</td>"
        + "<td id='opengain:" + id + "' class='right-align'>" + render(totals.open_gain) + "</td>"
        + "<td class='right-align'>" + render(totals.closed_gain) + "</td>"
        + "<td class='right-align'></td>"
        + "</tr>");

}

function deleteRow(rowid) {
    let row = document.getElementById(rowid);
    if (row !== null) {
        row.parentNode.removeChild(row);
    }
}

function closePosition(accountKey, instrumentKey) {
    let position = openBroker.getPosition(accountKey, instrumentKey);
    document.getElementById("order_quantity").value = -1 * position.quantity;
    let costBasis = render(position.cost / position.quantity);
    document.getElementById("order_price").value = costBasis;
    document.getElementById("order_instrument").value = position.instrument_key;
}

function computePositionRowId(accountKey, instrumentKey) {
    return "pos:" + accountKey + ":" + instrumentKey;
}

function marketCallback(market) {
    //console.log("market: " + JSON.stringify(market));

    let marketDataId = "marketData:" + market.instrument_key;
    deleteRow(marketDataId);
    let description = openBroker.getInstrumentDescription(market.instrument_key);
    let symbol = openBroker.getInstrumentSymbol(market.instrument_key);

    let text = "<tr id=" + marketDataId + ">";
    text += "<td title='" + description + "'>" + symbol + "</td>"

    text += "<td>";
    if (market.bid !== undefined && market.bid_size !== undefined) {
        text +=  market.bid_size + "@" + render(market.bid);
    } else {
        text += "-";
    }
    text += "</td>";

    text += "<td>";
    if (market.mark !== undefined) {
        text += render(market.mark);
    } else {
        text += "-";
    }
    text += "</td>";

    text += "<td>";
    if (market.ask !== undefined && market.ask_size !== undefined) {
        text += market.ask_size + "@" + render(market.ask);
    } else {
        text += "-";
    }
    text += "</td>";

    text += "<td>";
    if (market.last != undefined) {
        text += render(market.last);
    } else {
        text += "-";
    }
    text += "</td>";

    $("#markets_table").append(text);
}

function submitOrder() {
    let quantity = document.getElementById("order_quantity").value;
    let price = document.getElementById("order_price").value;
    let instrument_key = document.getElementById("order_instrument").value;
    let account_key = document.getElementById("order_account").value;

    document.getElementById("order_quantity").value = "";
    document.getElementById("order_price").value = "";

    openBroker.submitOrder(account_key, instrument_key, quantity, price);
}

function render(num) {
    if (num === undefined) {
        return "-";
    }
    else return num.toFixed(2);
}

function renderOrderStatus(orderStatus) {
    switch (orderStatus) {
        case "PendingCancel": return "Pending Cancel";
        default: return orderStatus;
    }
}

$( "#order_submit" ).click(() => submitOrder());

openBroker.instrumentCallback = instrumentCallback;
openBroker.accountCallback = accountCallback;
openBroker.positionCallback = positionCallback;
// openBroker.depthCallback = depthCallback;
// openBroker.lastTradeCallback = lastTradeCallback;
openBroker.orderStateCallback = orderStateCallback;
openBroker.balanceCallback = balanceCallback;
openBroker.totalsCallback = totalsCallback;
openBroker.marketCallback = marketCallback;
openBroker.start();