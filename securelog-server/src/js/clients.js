var xhr = new XMLHttpRequest();
xhr.open("GET", "/api/user/client/fetch_all");
xhr.setRequestHeader("Accept", "application/json");

xhr.onreadystatechange = function() {
    if (xhr.readyState == 4) {
        var clients = JSON.parse(xhr.responseText);
        var table = document.getElementById("client-table-body");

        var select = document.getElementById("delete-clientid");
        var select2 = document.getElementById("enabled-clientid");

        for (var i = 0; i < clients.length; i++) {

            var id = document.createElement("td");
            id.textContent = clients[i].id;

            var enabled = document.createElement("td");
            enabled.textContent = clients[i].enabled;

            var created = document.createElement("td");
            created.textContent = clients[i].created;

            var lastconnect = document.createElement("td");
            lastconnect.textContent = clients[i].lastconnect;

            var tr = document.createElement("tr");
            tr.appendChild(id);
            tr.appendChild(enabled);
            tr.appendChild(created);
            tr.appendChild(lastconnect);

            table.appendChild(tr);

            // fill select options
            var option = document.createElement("option");
            option.setAttribute("value", clients[i].id);
            option.textContent = clients[i].id + ": " + clients[i].name;
            
            select.appendChild(option.cloneNode(true));
            select2.appendChild(option);
        }
    }
}
xhr.send();