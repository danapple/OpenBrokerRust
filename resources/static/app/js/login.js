// const passwordField = document.getElementById("password");
// const togglePassword = document.getElementById(".password-toggle-icon i");

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

attachPasswordToggle(document.getElementById("login_password"), document.getElementById("login-password-toggle"));
attachPasswordToggle(document.getElementById("register_password"), document.getElementById("register-password-toggle"));

