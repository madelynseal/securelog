var params = new URLSearchParams(window.location.search);

var url = "/api/user/get_search_results?" + params.toString();

var xhr = new XMLHttpRequest();
xhr.open("GET", url);
xhr.setRequestHeader("Accept", "application/json");

xhr.onreadystatechange = function() {
    if (xhr.readyState == 4) {
        var results = JSON.parse(xhr.responseText);

        var container = document.getElementById("mycontainer");
        container.appendChild(document.createElement("br"));

        for (var i = 0; i < results.length; i++) {
            var result = results[i];

            var card = document.createElement("div");
            card.setAttribute("class", "card");

            var header = document.createElement("div");
            header.setAttribute("class", "card-header");
            header.textContent = result.search_id + ": " + result.search_name;
            card.appendChild(header);

            var body = document.createElement("div");
            body.setAttribute("class", "card-body");
            var h5 = document.createElement("h5");
            h5.textContent = "Time: " + result.started;

            body.appendChild(h5);

            for (var j = 0; j < result.found.length; j++) {
                body.appendChild(document.createElement("hr"));
                var para = document.createElement("p");
                para.textContent = result.found[j];
                body.appendChild(para);
            }

            card.appendChild(body);

            container.appendChild(card);
            container.appendChild(document.createElement("br"));
        }
    }
}
xhr.send();