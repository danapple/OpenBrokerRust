
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