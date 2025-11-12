import {OpenBroker} from "./openbroker.js";

let openBroker = new OpenBroker();

let positionsTable = new DataTable('#positions_table', {
    columns: [
        {title: 'Account', data: 'account', name: 'account', align: 'right'},
        {title: 'Instrument', data: 'instrument', name: 'instrument', align: 'right', orderData: [0, 1]},
        {title: 'Quantity', data: 'quantity', name: 'quantity', align: 'right', orderData: [0, 2]},
        {
            title: 'Cost Basis',
            data: 'cost_basis',
            name: 'cost_basis',
            align: 'right',
            defaultContent: '-',
            orderData: [0, 3]
        },
        {title: 'Cost', data: 'cost', name: 'cost', align: 'right', orderData: [0, 4]},
        {title: 'NetLiq', data: 'net_liq', align: 'right', name: 'net_liq', defaultContent: '-', orderData: [0, 5]},
        {
            title: 'Open Gain',
            data: 'open_gain',
            name: 'open_gain',
            align: 'right',
            defaultContent: '-',
            orderData: [0, 6]
        },
        {
            title: 'Open Gain %',
            data: 'open_gain_percent',
            name: 'open_gain_percent',
            align: 'right',
            defaultContent: '-',
            orderData: [0, 7]
        },
        {title: 'Closed Gain', data: 'closed_gain', name: 'closed_gain', align: 'right', orderData: [0, 8]},
        {title: 'Actions', data: 'actions', name: 'actions', align: 'right', orderData: [0, 9]},
        {title: 'RowId', data: 'row_id', name: 'row_id', align: 'right', visible: false},
    ],
    rowId: 'row_id',
    rowGroup: {
        dataSrc: 'account',
    }
});

let ordersTable = new DataTable('#orders_table', {
    columns: [
        {title: 'Account', data: 'account', name: 'account', align: 'right'},
        {title: 'Order Number', data: 'order_number', name: 'order_number', align: 'right', orderData: [0, 1]},
        {title: 'Instrument', data: 'instrument', name: 'instrument', align: 'right', orderData: [0, 2]},
        {title: 'Status', data: 'status', name: 'status', align: 'right', orderData: [0, 3]},
        {title: 'Side', data: 'side', name: 'side', align: 'right', orderData: [0, 4]},
        {title: 'Quantity', data: 'quantity', name: 'quantity', align: 'right', orderData: [0, 5]},
        {title: 'Price', data: 'price', name: 'price', align: 'right', orderData: [0, 6]},
        {title: 'Actions', data: 'actions', name: 'actions', align: 'right', orderData: [0, 7]},
        {title: 'RowId', data: 'row_id', name: 'row_id', align: 'right', visible: false},
    ],
    rowId: 'row_id',
    rowGroup: {
        dataSrc: 'account',
    },
    order: [[1, 'desc']]
});

let marketsTable = new DataTable('#markets_table', {
    columns: [
        {title: 'Instrument', data: 'instrument', name: 'instrument', align: 'right'},
        {title: 'Bid', data: 'bid', name: 'bid', align: 'right'},
        {title: 'Mark', data: 'mark', name: 'mark', align: 'right'},
        {title: 'Ask', data: 'ask', name: 'ask', align: 'right'},
        {title: 'Last', data: 'last', name: 'last', align: 'right'},
    ],
    rowId: 'row_id',
    rowGroup: {
        dataSrc: 'account',
    },
    order: [[1, 'desc']]
});

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
        "<td>" + account.account_number + "</td>"
        + "<td>" + account.account_name + "</td>"
        + "<td>" + account.nickname + "</td>");
    $("#account-data").show();

    let order_account_select = document.getElementById('order_account');

    let opt = document.createElement('option');
    opt.value = account.account_key;
    opt.innerHTML = account.account_number;
    order_account_select.appendChild(opt);
}

function getAccountSpan(accountKey) {
    let account = openBroker.getAccount(accountKey);
    return "<span title='" + account.account_name + "'>" + account.account_number + "</span>";
}

function getInstrumentSpan(instrumentKey) {
    let instrument = openBroker.getInstrument(instrumentKey);
    return "<span title='" + instrument.description + "'>" + instrument.symbol + "</span>";
}

function positionCallback(position) {
    // console.log("positionCallback: " + JSON.stringify(position));

    let accountDescription = getAccountSpan(position.account_key);
    let instrumentDescription = getInstrumentSpan(position.instrument_key);

    // console.log("position = " + JSON.stringify(position));
    let id = computePositionRowId(position.account_key, position.instrument_key);
    let closeButtonId = "close:" + id;

    let actions = "";
    if (position.quantity !== 0) {
        actions += "<button id='" + closeButtonId + "' class=\"btn btn-default\" type=\"submit\">Close</button>";
    }

    let element = document.getElementById(id);
    if (element === undefined || element === null) {
        let positionNode = positionsTable.row.add({
            account: accountDescription,
            instrument: instrumentDescription,
            quantity: position.quantity,
            cost_basis: render(position.cost_basis),
            cost: render(position.cost),
            net_liq: render(position.net_liq),
            open_gain: colorRender(position.open_gain),
            open_gain_percent: colorRender(position.open_gain_percent, "%"),
            closed_gain: colorRender(position.closed_gain),
            actions: actions,
            row_id: id
        }).draw().node();
        positionNode.setAttribute('id', id);
    } else {
        updateCell(positionsTable, id, 'quantity', render(position.quantity));
        updateCell(positionsTable, id, 'cost_basis', render(position.cost_basis));
        updateCell(positionsTable, id, 'cost', render(position.cost));
        updateCell(positionsTable, id, 'quantity', render(position.quantity));
        updateCell(positionsTable, id, 'net_liq', render(position.net_liq));
        updateCell(positionsTable, id, 'open_gain', colorRender(position.open_gain));
        updateCell(positionsTable, id, 'open_gain_percent', colorRender(position.open_gain_percent, "%"));
        updateCell(positionsTable, id, 'closed_gain', colorRender(position.closed_gain));
        updateCell(positionsTable, id, 'actions', actions);
        positionsTable.draw()
    }
    if (position.quantity !== 0) {
        document.getElementById(closeButtonId).addEventListener("click", () => {
            closePosition(position.account_key, position.instrument_key);
        })
    }
}

function updateCell(table, rowId, columnName, data) {
    var colIndex = table.column(columnName + ':name')[0][0];
    let cell = table.cell("#" + rowId, colIndex);
    cell.data(data);
}

function orderStateCallback(orderState) {
    // console.log("orderStateCallback: " + JSON.stringify(orderState));
    let id = "ord:" + orderState.order.account_key + ":" + orderState.order.ext_order_id;

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
    let accountDescription = getAccountSpan(orderState.order.account_key);
    let orderStatus = "<td class='right-align'";
    if (orderState.order_status === "Rejected" && orderState.reject_reason !== null) {
        orderStatus += " title='" + orderState.reject_reason + "'";
    }
    orderStatus += " >" + renderOrderStatus(orderState.order_status) + "</td>";

    let orderSide = orderState.order.quantity > 0 ? "Buy" : "Sell";

    let instrumentSpan =  "<span title='" + description + "'>" + symbol + "</span>";

    let element = document.getElementById(id);
    if (element === undefined || element === null) {
        let orderNode = ordersTable.row.add({
            account: accountDescription,
            order_number: orderState.order.order_number,
            instrument: instrumentSpan,
            status: orderStatus,
            side: orderSide,
            quantity: Math.abs(orderState.order.quantity),
            price: render(orderState.order.price),
            actions: actions,
            row_id: id
        }).draw().node();
        orderNode.setAttribute('id', id);
    } else {
        updateCell(ordersTable, id, 'status', orderStatus);
        updateCell(ordersTable, id, 'actions', actions);

        ordersTable.draw()
    }

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
    let needsDraw = false;
    let posbalanceid = "accbal:" + balance.account_key;
    let postotalsid = "acctot:" + balance.account_key;
    let accountDescription = getAccountSpan(balance.account_key);

    let balanceElement = document.getElementById(posbalanceid);
    if (balanceElement === undefined || balanceElement === null) {
        let cashNode = positionsTable.row.add({
            account: accountDescription,
            instrument: '-cash-',
            quantity: '',
            cost_basis: '',
            cost: '',
            net_liq: render(balance.cash),
            open_gain: '',
            open_gain_percent: '',
            closed_gain: '',
            actions: '',
            row_id: posbalanceid
        }).draw().node();
        cashNode.setAttribute("id", posbalanceid);
    } else {
        updateCell(positionsTable, posbalanceid, 'net_liq', render(balance.cash));
        needsDraw = true;
    }
    // console.log("totals.open_gain_percent = " + totals.open_gain_percent);

    // TODO maybe don't show sub-totals if there is only one account shown.
    let totalsElement = document.getElementById(postotalsid);
    if (totalsElement === undefined || totalsElement === null) {
        let subTotalsNode = positionsTable.row.add({
            account: accountDescription,
            instrument: '-sub-totals-',
            quantity: '',
            cost_basis: '',
            cost: render(totals.cost),
            net_liq: render(totals.net_liq),
            open_gain: colorRender(totals.open_gain),
            open_gain_percent: colorRender(totals.open_gain_percent, "%"),
            closed_gain: colorRender(totals.closed_gain),
            actions: '',
            row_id: postotalsid
        }).draw().node();
        subTotalsNode.setAttribute("id", postotalsid);
    } else {
        updateCell(positionsTable, postotalsid, 'cost', render(totals.cost));
        updateCell(positionsTable, postotalsid, 'net_liq', render(totals.net_liq));
        updateCell(positionsTable, postotalsid, 'open_gain', colorRender(totals.open_gain));
        updateCell(positionsTable, postotalsid, 'open_gain_percent', colorRender(totals.open_gain_percent, "%"));
        updateCell(positionsTable, postotalsid, 'closed_gain', colorRender(totals.closed_gain));
        needsDraw = true;
    }
    if (needsDraw) {
        positionsTable.draw()
    }
}

function totalsCallback(totals) {
    // console.log("totalsCallback: " + JSON.stringify(totals));

    let id = "totals:all";

    let totalsElement = document.getElementById(id);
    if (totalsElement === undefined || totalsElement === null) {
        let node = positionsTable.row.add({
            account: 'All',
            instrument: '-totals-',
            quantity: '',
            cost_basis: '',
            cost: render(totals.cost),
            net_liq: render(totals.net_liq),
            open_gain: colorRender(totals.open_gain),
            open_gain_percent: colorRender(totals.open_gain_percent, "%"),
            closed_gain: colorRender(totals.closed_gain),
            actions: '',
            row_id: id
        }).draw().node();
        node.setAttribute("id", id);
    } else {
        updateCell(positionsTable, id, 'cost', render(totals.cost));
        updateCell(positionsTable, id, 'net_liq', render(totals.net_liq));
        updateCell(positionsTable, id, 'open_gain', colorRender(totals.open_gain));
        updateCell(positionsTable, id, 'open_gain_percent', colorRender(totals.open_gain_percent, "%"));
        updateCell(positionsTable, id, 'closed_gain', colorRender(totals.closed_gain));
        positionsTable.draw()
    }
}

function closePosition(accountKey, instrumentKey) {
    let market = openBroker.getMarket(instrumentKey)
    let position = openBroker.getPosition(accountKey, instrumentKey);
    let orderSide = position.quantity > 0 ? "sell" : "buy";
    document.getElementById("order_quantity").value = Math.abs(position.quantity);
    document.getElementById("order_side").value = orderSide;

    let closingPrice = 0;
    if (orderSide === "sell") {
        closingPrice = market.bid;
    } else {
        closingPrice = market.ask;
    }

    document.getElementById("order_price").value = closingPrice;
    document.getElementById("order_instrument").value = position.instrument_key;
}

function computePositionRowId(accountKey, instrumentKey) {
    return "pos:" + accountKey + ":" + instrumentKey;
}

function marketCallback(market) {
    //console.log("market: " + JSON.stringify(market));

    let marketDataId = "marketData:" + market.instrument_key;
    let description = getInstrumentSpan(market.instrument_key);

    let bid = "-";
    if (market.bid !== undefined && market.bid_size !== undefined) {
        bid = market.bid_size + "@" + render(market.bid);
    }
    let mark = "-";
    if (market.mark !== undefined) {
        mark = render(market.mark);
    }
    let ask = "-";
    if (market.ask !== undefined && market.ask_size !== undefined) {
        ask = market.ask_size + "@" + render(market.ask);
    }
    let last = "-";
    if (market.last !== undefined) {
        last = market.last + "@" + render(market.last);
    }

    let marketDataElement = document.getElementById(marketDataId);
    if (marketDataElement === undefined || marketDataElement === null) {
        let node = marketsTable.row.add({
            instrument: description,
            bid: bid,
            mark: mark,
            ask: ask,
            last: last,
            row_id: marketDataId
        }).draw().node();
        node.setAttribute("id", marketDataId);
    } else {
        updateCell(marketsTable, marketDataId, 'bid', bid);
        updateCell(marketsTable, marketDataId, 'mark', mark);
        updateCell(marketsTable, marketDataId, 'ask', ask);
        updateCell(marketsTable, marketDataId, 'last', last);
        marketsTable.draw()
    }
}

function submitOrder() {
    let side = document.getElementById("order_side").value;
    let quantity = Math.abs(document.getElementById("order_quantity").value);
    let price = document.getElementById("order_price").value;
    let instrument_key = document.getElementById("order_instrument").value;
    let account_key = document.getElementById("order_account").value;
    if (side === "sell") {
        quantity *= -1;
    }
    if (quantity !== 0) {
        openBroker.submitOrder(account_key, instrument_key, quantity, price);
    }
}

function submitOrderCallback(status, text) {
    if (status === 200) {
        document.getElementById("order_quantity").value = "";
        document.getElementById("order_price").value = "";
    } else if (status === 412) {
        alert(text.reject_reason);
    }
}

function cancelOrderCallback(status, text) {
    if (status !== 200) {
        alert(status + " " + JSON.stringify(text));
    }
}

function render(num) {
    if (num === undefined || isNaN(num)) {
        return "-";
    }
    return num.toLocaleString('en', {
        useGrouping: true, minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    });
}

function colorRender(num, suffix) {
    let res = render(num);
    if (res === undefined) {
        return "-";
    }
    let cls = "";
    if (num > 0) {
        cls = "greencell";
    } else if (num < 0) {
        cls = "redcell";
    } else {
        cls = "right-align";
    }
    if (suffix !== undefined) {
        res += suffix;
    }
    // console.log("cls for " + res + " = " + cls);
    return "<span class='" + cls + "'>" + res + "</span>";
}

function renderOrderStatus(orderStatus) {
    switch (orderStatus) {
        case "PendingCancel":
            return "Pending Cancel";
        default:
            return orderStatus;
    }
}

$("#order_submit").click(() => submitOrder());

openBroker.instrumentCallback = instrumentCallback;
openBroker.accountCallback = accountCallback;
openBroker.positionCallback = positionCallback;
// openBroker.depthCallback = depthCallback;
// openBroker.lastTradeCallback = lastTradeCallback;
openBroker.orderStateCallback = orderStateCallback;
openBroker.balanceCallback = balanceCallback;
openBroker.totalsCallback = totalsCallback;
openBroker.marketCallback = marketCallback;
openBroker.submitOrderCallback = submitOrderCallback;
openBroker.cancelOrderCallback = cancelOrderCallback;
openBroker.start();
