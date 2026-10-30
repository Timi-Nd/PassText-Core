import concurrent.futures

from passtext import PassTextConfig, process_password


def test_prepared_password_repr_and_dict_do_not_expose_text():
    result = process_password("P@ssw0rd!", config=PassTextConfig(normalization="nfc"))

    assert "P@ssw0rd!" not in repr(result)
    assert "P@ssw0rd!" not in str(result.as_dict())
    assert result.as_dict()["normalization"] == "nfc"
    assert result.original_text == "P@ssw0rd!"
    assert result.processed_text == "P@ssw0rd!"


def test_repeated_evaluations_are_independent():
    config = PassTextConfig()
    first = process_password("éA🙂", config=config)
    second = process_password("ASCII", config=config)

    assert first.original_text == "éA🙂"
    assert second.original_text == "ASCII"
    assert first.metadata.original_codepoints == 3
    assert second.metadata.original_codepoints == 5
    assert first is not second


def test_concurrent_evaluations_are_safe():
    values = ["abc", "éé", "Γειά", "e\u0301", "🙂🙂"]

    def run(value: str):
        return process_password(value, config=PassTextConfig(normalization="nfc"))

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(run, values))

    assert len(results) == len(values)
    assert all(result.metadata.processed_codepoints >= 0 for result in results)
    assert all(result.original_text in values for result in results)
