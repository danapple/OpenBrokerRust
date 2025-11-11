// const passwordField = document.getElementById("password");
// const togglePassword = document.getElementById(".password-toggle-icon i");

function register(offerCode, actorName, emailAddress, password, callback) {
    let xhttp = makeXhttp("POST", "/register_ui", callback);
    let body = JSON.stringify({
        offer_code: offerCode,
        actor_name: actorName,
        email_address: emailAddress,
        password: password,
    });
    xhttp.send(body);
}

function login(emailAddress, password, callback) {
    let xhttp = makeXhttp("POST", "/login_ui", callback);
    let body = JSON.stringify({
        email_address: emailAddress,
        password: password,
    });
    xhttp.send(body);
}

function logout(callback) {
    let xhttp = makeXhttp("POST", "/logout", callback);
    let body = JSON.stringify({});
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

function attachPasswordToggle(fieldElement, toggleElement) {
    toggleElement.addEventListener("click", function () {
        if (fieldElement.type === "password") {
            fieldElement.type = "text";
            toggleElement.classList.remove("fa-eye");
            toggleElement.classList.add("fa-eye-slash");
        } else {
            fieldElement.type = "password";
            toggleElement.classList.remove("fa-eye-slash");
            toggleElement.classList.add("fa-eye");
        }
    });
}
