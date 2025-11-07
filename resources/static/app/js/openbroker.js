export class OpenBroker {
    instrumentCallback = null;
    accountCallback = null;
    positionCallback = null;
    depthCallback = null;
    lastTradeCallback = null;
    marketCallback = null;
    orderStateCallback = null;
    balanceCallback = null;
    totalsCallback = null;
    submitOrderCallback = null;
    cancelOrderCallback = null;

    constructor() {
        this.#stompClient = this.#buildStompClient();
    }

    start() {
        console.log('Starting OpenBroker')
        this.#getInstruments();
    }

    submitOrder(accountKey, instrumentKey, quantity, price) {
        let xhttp = this.#makeXhttp("POST", accountKey, undefined, this.submitOrderCallback);
        let body = JSON.stringify({
            price: Number(price),
            quantity: Number(quantity),
            legs: [
                {
                    ratio: Number(1),
                    instrument_key: instrumentKey
                }
            ]
        });
        xhttp.send(body);
    }

    cancelOrder(accountKey, extOrderId) {
        let xhttp = this.#makeXhttp("DELETE", accountKey, extOrderId, this.cancelOrderCallback);
        xhttp.send();
    }

    getAccount(accountKey) {
        return this.#getAccount(accountKey)['account'];
    }

    getPosition(accountKey, instrumentKey) {
        let positions = this.#getAccountArea(accountKey, 'positions');
        return positions[instrumentKey];
    }

    getInstrumentSymbol(instrument_key) {
        var instrument = this.#instruments[instrument_key];
        if (!instrument) {
            return "Id: " + instrument_key;
        }
        return instrument.symbol;
    }

    getInstrumentDescription(instrument_key) {
        var instrument = this.#instruments[instrument_key];
        if (!instrument) {
            return "Id: " + instrument_key;
        }
        return instrument.description;
    }

    /* Private below this line */

    #stompClient = null;
    #connected = false;
    #everConnected = false;
    #subscriptions = {};
    #instruments = {};
    #accountHolders = {};
    #marketData = {};
    #totals = {};

    #makeOrdersPath(accountKey, extra) {
        let path = "/accounts/" + accountKey + "/orders";
        if (extra !== undefined) {
            path += "/";
            path += extra;
        }
        return path;
    }

    #makeXhttp(method, accountKey, extra, callback) {
        let xhttp = new XMLHttpRequest();
        if (callback !== undefined && callback !== null) {
            xhttp.onreadystatechange = function () {
                if (this.readyState == 4) {
                    callback(this.status, JSON.parse(this.responseText));
                }
            };
        }
        let path = this.#makeOrdersPath(accountKey, extra);
        xhttp.open(method, path, true);
        xhttp.setRequestHeader("Content-type", "application/json");
        return xhttp;
    }

    #buildStompClient() {
        let stompClient = new StompJs.Client({
            webSocketFactory: function () {
                return new WebSocket("/ws");
            }
        });
        stompClient.onopen = function () {
            console.log("Stomp Client opened");
        }
        stompClient.onConnect = (frame) => {
            let delayMillis = 1;
            if (this.#everConnected) {
                delayMillis = (Math.random() * 5000).toFixed()
            }
            this.#setConnected();
            console.log('Connected');
            let openBroker = this;
            // console.warn("Delaying subscribe by " + delayMillis + " ms");
            setTimeout(function() {
                for (const [destination, func] of Object.entries(openBroker.#subscriptions)) {
                    openBroker.#sendsubscribe(destination, func);
                }
            }, delayMillis);
        };
        stompClient.onWebSocketError = (error) => {
            console.error('Error with websocket', error);
            this.#connected = false;
        };

        stompClient.onStompError = (frame) => {
            console.error('Broker reported error: ' + frame.headers['message']);
            console.error('Additional details: ' + frame.body);
        };
        return stompClient;
    }

    #subscribe(destination, func) {
        console.log('Subscribing to: ' + destination);
        this.#subscriptions[destination] = func;
        if (!this.#connected) {
            return;
        }
        this.#sendsubscribe(destination, func);
    }

    #subscribeAccount(accountKey, func) {
        let destination = '/accounts/' + accountKey + '/updates';
        this.#subscribe(destination, func);
    }

    #sendsubscribe(destination, func) {
        //  console.log('sendsubscribe to ' + destination);
        this.#stompClient.subscribe(destination, func);
        if (destination.startsWith('/accounts/')) {
            this.#stompClient.publish({destination: destination, body: JSON.stringify({request: "GET", scope: "balance"})});
            this.#stompClient.publish({destination: destination, body: JSON.stringify({request: "GET", scope: "positions"})});
            this.#stompClient.publish({destination: destination, body: JSON.stringify({request: "GET", scope: "orders"})});
        }
    }

    #setConnected() {
        this.#connected = true;
        this.#everConnected = true;
    }

    #computeMarket(instrumentKey) {
        let instrumentData = this.#marketData[instrumentKey];
         // console.log("Instrument data = " + JSON.stringify(instrumentData));
        if (instrumentData === undefined) {
            return undefined;
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
                    ask = sells0['price'];
                    askSize = sells0['quantity'];
                }
            }
        }

        let lastTrade = instrumentData['lastTrade'];
        if (lastTrade !== undefined) {
            last = lastTrade.price;
        }
        let mark = this.#computeMark(bid, ask, last);
        return {instrument_key: instrumentKey, bid: bid, bid_size: bidSize, ask: ask, ask_size: askSize, last: last, mark: mark};
    }

    #computeMark(bid, ask, last) {
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

    #getAccountArea(accountKey, area) {
        let accountHolder = this.#getAccount(accountKey);
        let accountArea = accountHolder[area];
        if (accountArea === undefined || accountArea === null) {
            accountHolder[area] = {}
        }
        return accountHolder[area];
    }

    #getAccount(accountKey) {
        if (this.#accountHolders[accountKey] === undefined || this.#accountHolders[accountKey] === null) {
            this.#accountHolders[accountKey] = {};
        }
        return this.#accountHolders[accountKey];
    }

    #updateAccountTotals(account_key) {
        let balance = this.#getAccountArea(account_key, 'balance');
        let accountTotals = this.#getAccountArea(account_key, 'totals');
        let positions = this.#getAccountArea(account_key, 'positions');

        let accountCost = 0;
        let accountOpenGain = 0;
        let accountClosedGain = 0;
        let accountNetLiq = 0;
        Object.values(positions).forEach(otherPosition => {
            accountCost += otherPosition['cost'];
            accountOpenGain += otherPosition['open_gain'];
            accountClosedGain += otherPosition['closed_gain'];
            accountNetLiq += otherPosition['net_liq'];
        });
        accountTotals['cost'] = accountCost;
        accountTotals['open_gain'] = accountOpenGain;
        accountTotals['closed_gain'] = accountClosedGain;
        accountTotals['net_liq'] = accountNetLiq;

        accountTotals['net_liq'] += balance['cash'];

        if (balance.account_key !== undefined && this.balanceCallback !== undefined && this.balanceCallback !== null) {
            this.balanceCallback(balance, accountTotals);
        }
    }

    #updateTotals() {
        let totalCost = 0;
        let totalOpenGain = 0;
        let totalClosedGain = 0;
        let totalNetLiq = 0;
        Object.values(this.#accountHolders).forEach(accountHolder => {
            let accountTotals = accountHolder['totals'];
            if (accountTotals === undefined) {
                accountTotals = {};
                accountHolder['totals'] = accountTotals;
            } else {
                totalCost += accountTotals['cost'];
                totalOpenGain += accountTotals['open_gain'];
                totalClosedGain += accountTotals['closed_gain'];
                totalNetLiq += accountTotals['net_liq'];
            }
        });

        this.#totals['cost'] = totalCost;
        this.#totals['open_gain'] = totalOpenGain;
        this.#totals['closed_gain'] = totalClosedGain;
        this.#totals['net_liq'] = totalNetLiq;

        if (this.totalsCallback !== undefined || this.totalsCallback !== null) {
            this.totalsCallback(this.#totals);
        }
    }

    #updatePositionWithMarket(position, market) {
        // console.log("updatePositionWithMarket accountKey " + position.account_key + ", instrumentKey " + position.instrument_key +
        //     " and market " + JSON.stringify(market));
        if (position === undefined || position === null) {
            console.log("No position for accountKey " + account_key + " and instrumentKey " + account_key);
            return;
        }
        if (market !== undefined && market.mark !== undefined) {
            let netLiq = market.mark * position.quantity;
            position['market'] = market;
            position['net_liq'] = netLiq;
            position['open_gain'] = netLiq - position['cost'];
        } else {
            position['market'] = undefined;
            position['net_liq'] = undefined;
            position['open_gain'] = undefined;
        }
        // console.log("updatePositionWithMarket position now " + JSON.stringify(position));

        if (this.positionCallback !== undefined && this.positionCallback !== null) {
            this.positionCallback(position);
        }
    }

    #updatePosition(position) {
        let market = this.#computeMarket(position.instrument_key);
        this.#updatePositionWithMarket(position, market);
        this.#updateAccountTotals(position.account_key);
        this.#updateTotals();
    }

    #updatePositions(instrumentKey, market) {
        for (let accountKey in this.#accountHolders) {
            let position = this.getPosition(accountKey, instrumentKey);
            let anyUpdates = false;
            if (position !== undefined && position !== null) {
                this.#updatePositionWithMarket(position, market);
                anyUpdates = true;
            }
            if (anyUpdates) {
                this.#updateAccountTotals(accountKey);
            }
        }
        this.#updateTotals();
    }

    #setMarketData(instrumentKey, category, data) {
        if (this.#marketData[instrumentKey] === undefined || this.#marketData[instrumentKey] === null) {
            this.#marketData[instrumentKey] = {};
        }
        let oldData = this.#marketData[instrumentKey][category];
        if (oldData !== undefined) {
            // console.log("Old market data version number: " + oldData.version_number +
            //     ", new market data version number: " + data.version_number);
            if (oldData.version_number >= data.version_number) {
                return false;
            }
        }
        this.#marketData[instrumentKey][category] = data;
      //  console.log("Market data = " + JSON.stringify(data));
        let market = this.#computeMarket(instrumentKey);
        this.#updatePositions(instrumentKey, market);
        if (this.marketCallback !== undefined && this.marketCallback !== null) {
            this.marketCallback(market);
        }
        return true;
    }

    #handleDepth(stompMessage) {
        let message = stompMessage.body;
         // console.log('Got depth message: ' + message);
        let depth = JSON.parse(message);
        if (this.depthCallback !== undefined && this.depthCallback !== null) {
            this.depthCallback(depth);
        }
        this.#setMarketData(depth.instrument_key, 'depth', depth);
    }

    #handleLastTrade(stompMessage) {
        let message = stompMessage.body;
          // console.log('Got last trade message: ' + message);
        let lastTrade = JSON.parse(message);
        if (this.lastTradeCallback !== undefined && this.lastTradeCallback !== null) {
            this.lastTradeCallback(lastTrade);
        }
        this.#setMarketData(lastTrade.instrument_key, 'lastTrade', lastTrade);
    }

    #handleBalance(balance) {
        if (balance !== undefined && balance !== null) {
            let accountHolder = this.#getAccount(balance.account_key);
            let oldBalance = accountHolder['balance'];
            if (oldBalance !== undefined) {
                if (oldBalance.version_number >= balance.version_number) {
                    return false;
                }
            }
            accountHolder['balance'] = balance;
            this.#updateAccountTotals(balance.account_key);
            this.#updateTotals();
            if (this.balanceCallback !== undefined && this.balanceCallback !== null) {
                let accountTotals = this.#getAccountArea(balance.account_key, 'totals');
                this.balanceCallback(balance, accountTotals);
            }
        }
    }

    #handlePosition(position) {
        if (position !== undefined && position !== null) {
            let accountPositions = this.#getAccountArea(position.account_key, 'positions');
            let oldPosition = accountPositions[position.instrument_key];
            if (oldPosition !== undefined && oldPosition !== null) {
                if (oldPosition.version_number >= position.version_number) {
                    // console.log("Old position version number: " + oldPosition.version_number +
                    //     ", new position version number: " + position.version_number +
                    //     ", skipping update");
                    return;
                }
            }
            accountPositions[position.instrument_key] = position;
            if (position.quantity !== 0) {
                position['cost_basis'] = (position.cost / position.quantity);
            }
            this.#updatePosition(position);
        }
    }

    #handleOrderState(orderState) {
        if (orderState !== undefined && orderState !== null) {
            let accountOrderStates = this.#getAccountArea(orderState.order.account_key, 'orders');
            let oldOrderState = accountOrderStates[orderState.order.ext_order_id];
            if (oldOrderState !== undefined) {
                // console.log("Old order state version number: " + oldOrderState.version_number +
                //     ", new order state  version number: " + orderState.version_number);
                if (oldOrderState.version_number >= orderState.version_number) {
                    // console.log("Old order state version number: " + oldOrderState.version_number +
                    //     ", new order state version number: " + orderState.version_number +
                    //     ", skipping update");
                    return;
                }
            }
            accountOrderStates[orderState.order.ext_order_id] = orderState;
            if (this.orderStateCallback !== undefined && this.orderStateCallback !== null) {
                this.orderStateCallback(orderState);
            }
        }
    }

    #handleUpdate(stompMessage) {
        let message = stompMessage.body;
        let account_update = JSON.parse(message);
        if (account_update === undefined || account_update === null) {
            return
        }
        if (account_update.balance !== undefined && account_update.balance !== null) {
            this.#handleBalance(account_update['balance']);
        }
        if (account_update.position !== undefined && account_update.position !== null) {
            this.#handlePosition(account_update.position);
        }
        if (account_update.order_state !== undefined && account_update.order_state !== null) {
            this.#handleOrderState(account_update.order_state);
        }
    }

    #processAccounts(account_data) {
          // console.log('Got accounts:' + JSON.stringify(account_data));
        Object.values(account_data).forEach((account) => {
            let accountHolder = this.#getAccount(account.account_key);
            accountHolder['account'] = account;
            this.#subscribeAccount(account.account_key, (foo) => {
                this.#handleUpdate(foo);
            });
            if (this.accountCallback !== undefined && this.accountCallback !== null) {
                this.accountCallback(account);
            }
        });
    }

    #getAccounts() {
        let xhttp = new XMLHttpRequest();
        let openBroker = this;

        xhttp.onreadystatechange = function () {
            if (this.readyState == 4 && this.status == 200) {
                openBroker.#processAccounts(JSON.parse(this.responseText));
            }
        };
        xhttp.open("GET", "/accounts", true);
        xhttp.setRequestHeader("Content-type", "application/json");
        xhttp.send();
    }

    #processInstruments(instrument_data) {
        //  console.log('Got instruments:' + JSON.stringify(instrument_data));
        this.#instruments = instrument_data;

        Object.values(this.#instruments).forEach((instrument) => {
            //      console.log('instrument: ' + JSON.stringify(instrument));
            if (instrument.status != 'Active' || instrument.expiration_time < Date.now()) {
                return;
            }
            if (this.instrumentCallback !== null && this.instrumentCallback !== undefined) {
                this.instrumentCallback(instrument);
            }
            this.#subscribe('/markets/' + instrument.instrument_key + '/depth', (foo) => {
                this.#handleDepth(foo);
            });
            this.#subscribe('/markets/' + instrument.instrument_key + '/last_trade', (foo) => {
                this.#handleLastTrade(foo);
            });
        });
        this.#getAccounts();
        this.#stompClient.activate();
    }

    #getInstruments() {
        let xhttp = new XMLHttpRequest();
        let openBroker = this;
        xhttp.onreadystatechange = function () {
            if (this.readyState == 4 && this.status == 200) {
                openBroker.#processInstruments(JSON.parse(this.responseText));
            }
        };
        xhttp.open("GET", "/instruments", true);
        xhttp.setRequestHeader("Content-type", "application/json");
        xhttp.send();
    }
}