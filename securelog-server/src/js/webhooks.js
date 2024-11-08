var xhr = new XMLHttpRequest();
xhr.open("GET", "/api/user/webhooks/fetch");
xhr.setRequestHeader("Accept", "application/json");

xhr.onreadystatechange = function() {
    if (xhr.readyState == 4) {
        var clients = JSON.parse(xhr.responseText);

        var tbody = document.getElementById("tbody-webhooks");
        var select = document.getElementById("webhook-delete-select");

        for (var i = 0; i < clients.length; i++) {
            var client = clients[i];

            var name = document.createElement("td");
            name.textContent = client.name;

            var url = document.createElement("td");
            url.textContent = client.url;

            var username = document.createElement("td");
            username.textContent = client.username;

            var tr = document.createElement("tr");
            tr.appendChild(name);
            tr.appendChild(url);
            tr.appendChild(username);
            tbody.appendChild(tr);

            var option = document.createElement("option");
            option.setAttribute("value", client.name);
            option.textContent = client.name;
            select.appendChild(option);
        }

    }
}
xhr.send();

