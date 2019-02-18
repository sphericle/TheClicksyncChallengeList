<div class='panel fade js-scroll-anim' data-anim='fade'>

# Pagination and Filtering{id=pagination}

Some endpoints in the pointercrate API support or require pagination due to the potentially huge amount of data they can return.
This mostly applies to the endpoints that return lists of objects, like [`GET /records/`](/documentation/records/#get-records).

Objects returned by endpoints supporting pagination are totally ordered by an ID field, which is specified in the endpoint's documentation.

If an endpoint supports pagination, it's documentation will contain a notice similar to this one:

<div class='info-green'>
<b>Pagination:</b><br>
This endpoint supports [pagination and filtering](#pagination) via query parameters. Please see the documentation on pagination for information
on the additional request and response headers.
</div>

## Pagination Query Parameters

Pagination is done via specific query parameters, which tell pointercrate which part of the result set to return.

Note that there is no way to get the total amount of pages, as both page bounds and size can be chosen abitrarily.

| Query Parameter | Description                                                                                         | Default                               |
| --------------- | --------------------------------------------------------------------------------------------------- | ------------------------------------- |
| limit           | The maximum amount of object to return. Must lie between `1` and `100`                              | 50                                    |
| after           | The id of the last object on the previous page, thus specifying the start point of the current page | The very first item in the result set |
| before          | The id of the first object on the next page, thus specifying the end point of the current page      | The very last item in the result set  |

## Pagination Response Headers

Paginatable endpoints provide the `Links` header to simply access to the next, previous, first and last page, using the `limit` set on the request.
The header is set to a comma-seperated
list of links in the form `<[link]>; rel=[page]`, where page is one of `next`, `prev`, `first` or `last`.

Note that the `next` and `prev` links are only provided if there actually is a next or previous page of results respectively. The `first` and `last` links are always provided.

## Filtering

Most endpoints that support pagination also support filtering their results.

If this is supported, the documentation specifies the filterable fields for a given endpoint.
It is then possible to specify conditions in the query string, which the returned objects must meet.

There are two ways of filtering the result set:

// TODO: reimplement comma list of values for equality

// TODO: maybe implement sorting?

- **Filtering by equality**: The objects returned can be filtered by a specific field's value by specifying the field and a value in the query string, i.e. `/api/v1/records?id=54`
- **Filtering by inequality**: The objects returned can be filtered by whether a field is smaller/greater than a specific value by specifying the field,
  suffixed with either `__lt` or `__gt`, and the value to check for inequality against in the query string, i.e. `/api/v1/records?id__gt=20`. Note that this doesn't work for all fields (since a lexicographical filtering on the record status hardly seems useful)

Multiple conditions can be combined, i.e. `/api/v1/records?status=APPROVED&id__gt=200&id__lt=1000`.

### Errors:

These error conditions can occur at any endpoint supporting pagination and are thus not listed specifically for each of them.

// TODO: this error handling needs to be reimplemented

| Status code | Error code | Description                                                     |
| ----------- | ---------- | --------------------------------------------------------------- |
| 422         | 42207      | The `limit` parameter is smaller than `1` or greater than `100` |
| 422         | 42208      | The `after` parameter is greater than the `before` parameter    |
| 422         | 42209      | The `after` parameter is smaller than `0`                       |

</div>