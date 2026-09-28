UPDATE `notes` SET `title` = TRIM(SUBSTR(`title`, 1, INSTR(`title`, CHAR(10)) - 1), ' ' || CHAR(9) || CHAR(10) || CHAR(13)) WHERE INSTR(`title`, CHAR(10)) > 0;
