
function createOfferCallback(status, text) {
    if (status === 200) {
        document.getElementById("create_offer_success_message").innerHTML = "Offer created";
        document.getElementById("offer_code").innerHTML = "";
        document.getElementById("description").innerHTML = "";
        document.getElementById("expiration").value = "";
    } else {
        document.getElementById("create_offer_failure_message").innerHTML
            = "Creation rejected with status " + status;
    }
}

function getMinExpirationDateTime() {
    return new Date(Date.now() + 3600 * 1000).toISOString().slice(0, -5);
}

function createOffer() {
    document.getElementById("create_offer_success_message").innerHTML = "";
    document.getElementById("create_offer_failure_message").innerHTML = "";

    let offer_code = document.getElementById("offer_code").value;
    let description = document.getElementById("description").value;
    let expiration = document.getElementById("expiration").value;
    let expiration_date = new Date(expiration);
    let expiration_millis = expiration_date.getTime();
    // console.log("Expiration getTime = " + expiration_millis);
    let xhttp = makeXhttp("POST", "/admin/offer", createOfferCallback);
    let body = JSON.stringify({
        code: offer_code,
        description: description,
        expiration_time: expiration_millis,
    });
    xhttp.send(body);
}

function makeXhttp(method, path, callback) {
    let xhttp = new XMLHttpRequest();
    if (callback !== undefined && callback !== null) {
        xhttp.onreadystatechange = function () {
            if (this.readyState == 4) {
                callback(this.status);
            }
        };
    }
    xhttp.open(method, path, true);
    xhttp.setRequestHeader("Content-type", "application/json");
    return xhttp;
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

let resetDateTime = getMinExpirationDateTime() ;
console.log("resetDateTime = " + resetDateTime);
document.getElementById("expiration").min = resetDateTime;

$("#create_offer").click(() => createOffer());
