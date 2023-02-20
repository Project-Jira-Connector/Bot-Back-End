# Robot-for-Telkom-s-Atlassian-Cloud-Platform

```http
  POST /robots
```
| Parameter | Type     | Requirement | Description                |
| :-------- | :------- | :---------- | :------------------------- |
| `name` | `String` | **Required** | |
| `description` | `String` | **Required** | |
| `platformEmail` | `String` | **Required** | |
| `platformApiKey` | `String` | **Required** | |
| `platformType` | `Enum` | **Required** | |
| `cloudSessionToken` | `String` | **Required** | |
| `active` | `bool` | **Required** | |
| `schedule` | `i64` | **Required** | |
| `lastActive` | `i64` | **Required** | |
| `checkActiveStatus` | `bool` | **Required** | |
| `checkDoubleEmail` | `bool` | **Required** | |
| `checkDoubleName` | `bool` | **Required** | |

```http
  GET /robots
```
| Parameter | Type     | Requirement | Description                |
| :-------- | :------- | :---------- | :------------------------- |
| `_id` | `ObjecId` | *Optional* | |
| `name` | `String` | *Optional* | |
| `description` | `String` | *Optional* | |
| `platformEmail` | `String` | *Optional* | |
| `platformApiKey` | `String` | *Optional* | |
| `platformType` | `Enum` | *Optional* | |
| `cloudSessionToken` | `String` | *Optional* | |
| `active` | `bool` | *Optional* | |
| `schedule` | `i64` | *Optional* | |
| `lastActive` | `i64` | *Optional* | |
| `checkActiveStatus` | `bool` | *Optional* | |
| `checkDoubleEmail` | `bool` | *Optional* | |
| `checkDoubleName` | `bool` | *Optional* | |

```http
  PATCH /robots
```
| Parameter | Type     | Requirement | Description                |
| :-------- | :------- | :---------- | :------------------------- |
| `_id` | `ObjecId` | **Required** | |
| `name` | `String` | *Optional* | |
| `description` | `String` | *Optional* | |
| `platformEmail` | `String` | *Optional* | |
| `platformApiKey` | `String` | *Optional* | |
| `platformType` | `Enum` | *Optional* | |
| `cloudSessionToken` | `String` | *Optional* | |
| `active` | `bool` | *Optional* | |
| `schedule` | `i64` | *Optional* | |
| `lastActive` | `i64` | *Optional* | |
| `checkActiveStatus` | `bool` | *Optional* | |
| `checkDoubleEmail` | `bool` | *Optional* | |
| `checkDoubleName` | `bool` | *Optional* | |

```http
  DELETE /robots
```
| Parameter | Type     | Requirement | Description                |
| :-------- | :------- | :---------- | :------------------------- |
| `_id` | `ObjecId` | **Required** | |