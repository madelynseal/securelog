var xhr = new XMLHttpRequest();
xhr.open("GET", "/api/user/client/fetch_all");
xhr.setRequestHeader("Accept", "application/json");

xhr.onreadystatechange = function () {
    if (xhr.readyState == 4) {
        var clients = JSON.parse(xhr.responseText);

        var select = document.getElementById("client-select");

        for (var i = 0; i < clients.length; i++) {
            var option = document.createElement("option");
            option.setAttribute("value", clients[i].id);
            option.textContent = clients[i].id + ": " + clients[i].name;

            select.appendChild(option);
        }
    }
}
xhr.send();