var xhr = new XMLHttpRequest();
xhr.open("GET", "/api/user/get_searches");
xhr.setRequestHeader("Accept", "application/json");

xhr.onreadystatechange = function() {
    if (xhr.readyState == 4) {
        var searches = JSON.parse(xhr.responseText);

        var body = document.getElementById("searches-tbody");

        var delete_select = document.getElementById("delete-search-id");

        for (var i = 0; i < searches.length; i++) {
            var search = searches[i];

            var id = document.createElement("td");
            id.textContent = search.id;
            
            var name = document.createElement("td");
            name.textContent = search.name;

            var stype = document.createElement("td");
            stype.textContent = search.stype;

            var text = document.createElement("td");
            text.textContent = search.search;

            var locations = document.createElement("td");
            locations.textContent = search.locations.join(',');


            var tr = document.createElement("tr");

            tr.appendChild(id);
            tr.appendChild(name);
            tr.appendChild(stype);
            tr.appendChild(text);
            tr.appendChild(locations);

            body.appendChild(tr);

            var option = document.createElement("option");
            option.setAttribute("value", search.id);
            option.textContent = search.id + ": " + search.name;
            delete_select.appendChild(option);
        }
        
    }
}
xhr.send();