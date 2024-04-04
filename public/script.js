const ul = document.querySelector('ul');
const form = document.querySelector('form');
let items = [];
function updateItems() {
  fetch("/api/all").then(r => r.json()).then(json => {
    items = json;
    let children = [];
    for (const item of items) {
      const newNode = document.createElement('li');
      newNode.innerText = item;
      children.push(newNode);
    }
    ul.replaceChildren(...children);
  })
}
updateItems()
const input = document.querySelector('input');
form.addEventListener('submit', (e) => {
  e.preventDefault()
  if (!input.value) {return}
  fetch("/api/add", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(input.value)
  });
  input.value = "";
  updateItems()
})