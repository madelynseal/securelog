const params = new URLSearchParams(window.location.search);

const redirect = params.get("redirect");

var element = document.getElementById("redirect-input");
element.setAttribute("value", redirect);