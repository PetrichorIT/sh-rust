/**
 *
 * @param {string} itemName
 * @param {string} itemValue
 */
export function setStoredValue(itemName: string, itemValue: any) {
  // Store as Cookie
  const d = new Date();
  d.setTime(d.getTime() + 24 * 60 * 60 * 1000);
  const expires = "expires=" + d.toUTCString();
  document.cookie = itemName + "=" + itemValue + ";" + expires + ";path=/";

  // Store in sessionStorage
  sessionStorage.setItem(itemName, itemValue);
}

/**
 *
 * @param {string} itemName
 */
export function getStoredValue(itemName: string): any {
  let value = sessionStorage.getItem(itemName);
  if (value === null) {
    const name = itemName + "=";
    const decodedCookie = decodeURIComponent(document.cookie);
    const ca = decodedCookie.split(";");
    for (let i = 0; i < ca.length; i++) {
      let c = ca[i];
      while (c.charAt(0) == " ") {
        c = c.substring(1);
      }
      if (c.indexOf(name) == 0) {
        value = c.substring(name.length, c.length);
        setStoredValue(itemName, value);
        break;
      }
    }
  }
  return value;
}
